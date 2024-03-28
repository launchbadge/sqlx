use crate::{
    error::Error,
    io::Encode,
    message::{CopyData, CopyDone, CopyResponse, MessageFormat, Query},
    PgConnectOptions, PgPool, PgPoolOptions, PgReplicationMode, Result,
};
use futures_util::future::Either;
use futures_util::stream::Stream;
use sqlx_core::bytes::Bytes;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct PgReplicationPool(PgPool);

impl PgReplicationPool {
    pub async fn connect(url: &str, mode: PgReplicationMode) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .max_lifetime(None)
            .idle_timeout(None)
            .connect(url)
            .await?;
        Ok(Self::from_pool(pool, mode))
    }

    pub fn from_pool(pool: PgPool, mode: PgReplicationMode) -> Self {
        let pool_options = pool.options().clone();
        let connect_options =
            <PgConnectOptions as Clone>::clone(&pool.connect_options()).replication_mode(mode);

        Self(pool_options.parent(pool).connect_lazy_with(connect_options))
    }

    /// Open a duplex connection allowing high-speed bulk data transfer to and from the server.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sqlx::postgres::{
    ///     PgReplicationPool, PgReplicationMode, PgReplication
    /// };
    /// use futures_util::stream::StreamExt;
    /// # #[cfg(feature = "_rt")]
    /// # sqlx::__rt::test_block_on(async move {
    /// let pool = PgReplicationPool::connect("0.0.0.0", PgReplicationMode::Logical)
    ///     .await
    ///     .expect("failed to connect to postgres");
    ///
    /// let query = format!(
    ///     r#"START_REPLICATION SLOT "{}" LOGICAL {} ("proto_version" '1', "publication_names" '{}')"#,
    ///     "test_slot", "0/1573178", "test_publication",
    /// );
    /// let PgReplication {sender, receiver} = pool.start_replication(query.as_str())
    ///     .await
    ///     .expect("start replication");
    /// // Read data from the server.
    /// while let Some(data) = receiver.next().await {
    ///     println!("data: {:?}", data);
    ///     // And send some back (e.g. keepalive).
    ///     sender.send(Vec::new()).await?;
    /// }
    /// // Connection closed.
    /// # Result::<(), Error>::Ok(())
    /// # }).unwrap();
    /// ```
    pub async fn start_replication(&self, statement: &str) -> Result<PgReplication> {
        // Setup upstream/downstream channels.
        let (recv_tx, recv_rx) = flume::bounded(1);
        let (send_tx, send_rx) = flume::bounded(1);

        crate::rt::spawn({
            let pool = self.clone();
            async move {
                if let Err(err) = copy_both_handler(pool, recv_tx.clone(), send_rx).await {
                    let _ignored = recv_tx.send_async(Err(err)).await;
                }
            }
        });

        // Execute the given statement to switch into CopyBoth mode.
        let mut buf = Vec::new();
        Query(statement).encode(&mut buf);
        send_tx
            .send_async(PgCopyBothCommand::Begin(buf))
            .await
            .map_err(|_err| Error::WorkerCrashed)?;

        Ok(PgReplication {
            sender: PgCopyBothSender(send_tx),
            receiver: PgCopyBothReceiver(recv_rx.into_stream()),
        })
    }
}

enum PgCopyBothCommand {
    Begin(Vec<u8>),
    CopyData(Vec<u8>),
    CopyDone { from_client: bool },
}

pub struct PgCopyBothSender(flume::Sender<PgCopyBothCommand>);
pub struct PgCopyBothReceiver(flume::r#async::RecvStream<'static, Result<Bytes>>);

pub struct PgReplication {
    pub receiver: PgCopyBothReceiver,
    pub sender: PgCopyBothSender,
}

impl PgCopyBothSender {
    /// Send a chunk of `COPY` data.
    pub async fn send(&self, data: impl Into<Vec<u8>>) -> Result<()> {
        self.0
            .send_async(PgCopyBothCommand::CopyData(data.into()))
            .await
            .map_err(|_err| Error::WorkerCrashed)?;

        Ok(())
    }

    /// Signal that the CopyBoth mode is complete.
    pub async fn finish(self) -> Result<()> {
        self.0
            .send_async(PgCopyBothCommand::CopyDone { from_client: true })
            .await
            .map_err(|_err| Error::WorkerCrashed)?;

        Ok(())
    }
}

impl Stream for PgCopyBothReceiver {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}

async fn copy_both_handler(
    pool: PgReplicationPool,
    recv_tx: flume::Sender<Result<Bytes>>,
    send_rx: flume::Receiver<PgCopyBothCommand>,
) -> Result<()> {
    let mut has_started = false;
    let mut conn = pool.0.acquire().await?;
    conn.wait_until_ready().await?;

    loop {
        // Wait for either incoming data or a message to send.
        let command = match futures_util::future::select(
            std::pin::pin!(conn.stream.recv()),
            std::pin::pin!(send_rx.recv_async()),
        )
        .await
        {
            Either::Left((data, _)) => {
                let message = data?;
                match message.format {
                    MessageFormat::CopyData => {
                        recv_tx
                            .send_async(message.decode::<CopyData<Bytes>>().map(|x| x.0))
                            .await
                            .map_err(|_err| Error::WorkerCrashed)?;
                        None
                    }
                    // Server is done sending data, close our side.
                    MessageFormat::CopyDone => {
                        let _ = message.decode::<CopyDone>()?;
                        Some(PgCopyBothCommand::CopyDone { from_client: false })
                    }
                    _ => {
                        return Err(err_protocol!(
                            "unexpected message format during copy out: {:?}",
                            message.format
                        ))
                    }
                }
            }
            // This only errors if the consumer has been dropped.
            // There is no reason to continue.
            Either::Right((command, _)) => Some(command.map_err(|_| Error::WorkerCrashed)?),
        };

        if let Some(command) = command {
            match command {
                // Start the stream.
                PgCopyBothCommand::Begin(buf) => {
                    if has_started {
                        return Err(err_protocol!("Copy-Both mode already initiated"));
                    }
                    conn.stream.send(buf.as_slice()).await?;
                    // Consume the server response.
                    conn.stream
                        .recv_expect::<CopyResponse>(MessageFormat::CopyBothResponse)
                        .await?;
                    has_started = true;
                }
                // Send data to the server.
                PgCopyBothCommand::CopyData(data) => {
                    if !has_started {
                        return Err(err_protocol!("connection hasn't been started"));
                    }
                    conn.stream.send(CopyData(data)).await?;
                }

                // Grafeceful shutdown of the stream.
                PgCopyBothCommand::CopyDone { from_client } => {
                    if !has_started {
                        return Err(err_protocol!("connection hasn't been started"));
                    }
                    conn.stream.send(CopyDone).await?;
                    // If we are the first to send CopyDone, wait for the server to send his own.
                    if from_client {
                        conn.stream.recv_expect(MessageFormat::CopyDone).await?;
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}
