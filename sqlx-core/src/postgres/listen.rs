use std::fmt::{self, Debug};
use std::io;

use async_stream::try_stream;
use futures_channel::mpsc;
use futures_core::future::BoxFuture;
use futures_core::stream::Stream;

use crate::describe::Describe;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::pool::{Pool, PoolConnection};
use crate::postgres::protocol::{Message, NotificationResponse};
use crate::postgres::{PgConnection, PgCursor, Postgres};

/// A stream of asynchronous notifications from Postgres.
///
/// This listener will auto-reconnect. If the active
/// connection being used ever dies, this listener will detect that event, create a
/// new connection, will re-subscribe to all of the originally specified channels, and will resume
/// operations as normal.
pub struct PgListener {
    pool: Pool<PgConnection>,
    connection: Option<PoolConnection<PgConnection>>,
    buffer_rx: mpsc::UnboundedReceiver<NotificationResponse<'static>>,
    buffer_tx: Option<mpsc::UnboundedSender<NotificationResponse<'static>>>,
    channels: Vec<String>,
}

/// An asynchronous notification from Postgres.
pub struct PgNotification<'c>(NotificationResponse<'c>);

impl PgListener {
    pub async fn new(url: &str) -> crate::Result<Self> {
        // Create a pool of 1 without timeouts (as they don't apply here)
        // We only use the pool to handle re-connections
        let pool = Pool::<PgConnection>::builder()
            .max_size(1)
            .max_lifetime(None)
            .idle_timeout(None)
            .build(url)
            .await?;

        Self::from_pool(&pool).await
    }

    pub async fn from_pool(pool: &Pool<PgConnection>) -> crate::Result<Self> {
        // Pull out an initial connection
        let mut connection = pool.acquire().await?;

        // Setup a notification buffer
        let (sender, receiver) = mpsc::unbounded();
        connection.stream.notifications = Some(sender);

        Ok(Self {
            pool: pool.clone(),
            connection: Some(connection),
            buffer_rx: receiver,
            buffer_tx: None,
            channels: Vec::new(),
        })
    }

    /// Starts listening for notifications on a channel.
    pub async fn listen(&mut self, channel: &str) -> crate::Result<()> {
        self.connection()
            .execute(&*format!("LISTEN {}", ident(channel)))
            .await?;

        self.channels.push(channel.to_owned());

        Ok(())
    }

    /// Starts listening for notifications on all channels.
    pub async fn listen_all(
        &mut self,
        channels: impl IntoIterator<Item = &str>,
    ) -> crate::Result<()> {
        let beg = self.channels.len();
        self.channels.extend(channels.into_iter().map(|s| s.into()));

        self.connection
            .as_mut()
            .unwrap()
            .execute(&*build_listen_all_query(&self.channels[beg..]))
            .await?;

        Ok(())
    }

    /// Stops listening for notifications on a channel.
    pub async fn unlisten(&mut self, channel: &str) -> crate::Result<()> {
        self.connection()
            .execute(&*format!("UNLISTEN {}", ident(channel)))
            .await?;

        if let Some(pos) = self.channels.iter().position(|s| s == channel) {
            self.channels.remove(pos);
        }

        Ok(())
    }

    /// Stops listening for notifications on all channels.
    pub async fn unlisten_all(&mut self) -> crate::Result<()> {
        self.connection().execute("UNLISTEN *").await?;

        self.channels.clear();

        Ok(())
    }

    #[inline]
    async fn connect_if_needed(&mut self) -> crate::Result<()> {
        if let None = self.connection {
            let mut connection = self.pool.acquire().await?;
            connection.stream.notifications = self.buffer_tx.take();

            connection
                .execute(&*build_listen_all_query(&self.channels))
                .await?;

            self.connection = Some(connection);
        }

        Ok(())
    }

    #[inline]
    fn connection(&mut self) -> &mut PgConnection {
        self.connection.as_mut().unwrap()
    }

    /// Receives the next notification available from any of the subscribed channels.
    pub async fn recv(&mut self) -> crate::Result<PgNotification<'_>> {
        // Flush the buffer first, if anything
        // This would only fill up if this listener is used as a connection
        if let Ok(Some(notification)) = self.buffer_rx.try_next() {
            return Ok(PgNotification(notification));
        }

        loop {
            // Ensure we have an active connection to work with.
            self.connect_if_needed().await?;

            match self.connection().stream.read().await {
                // We've received an async notification, return it.
                Ok(Message::NotificationResponse) => {
                    let notification =
                        NotificationResponse::read(self.connection().stream.buffer())?;

                    return Ok(PgNotification(notification));
                }

                // Mark the connection as ready for another query
                Ok(Message::ReadyForQuery) => {
                    self.connection().is_ready = true;
                }

                // Ignore unexpected messages
                Ok(_) => {}

                // The connection is dead, ensure that it is dropped,
                // update self state, and loop to try again.
                Err(crate::Error::Io(err)) if err.kind() == io::ErrorKind::ConnectionAborted => {
                    self.buffer_tx = self.connection().stream.notifications.take();
                    self.connection = None;
                }

                // Forward other errors
                Err(error) => {
                    return Err(error);
                }
            }
        }
    }

    /// Consume this listener, returning a `Stream` of notifications.
    pub fn into_stream(
        mut self,
    ) -> impl Stream<Item = crate::Result<PgNotification<'static>>> + Unpin {
        Box::pin(try_stream! {
            loop {
                let notification = self.recv().await?;
                yield notification.into_owned();
            }
        })
    }
}

impl Executor for PgListener {
    type Database = Postgres;

    fn execute<'e, 'q: 'e, 'c: 'e, E: 'e>(
        &'c mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Execute<'q, Self::Database>,
    {
        self.connection().execute(query)
    }

    fn fetch<'q, E>(&mut self, query: E) -> PgCursor<'_, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        self.connection().fetch(query)
    }

    #[doc(hidden)]
    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>,
    {
        self.connection().describe(query)
    }
}

impl<'c> RefExecutor<'c> for &'c mut PgListener {
    type Database = Postgres;

    fn fetch_by_ref<'q, E>(self, query: E) -> PgCursor<'c, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        self.connection().fetch_by_ref(query)
    }
}

impl PgNotification<'_> {
    /// The process ID of the notifying backend process.
    #[inline]
    pub fn process_id(&self) -> u32 {
        self.0.process_id
    }

    /// The channel that the notify has been raised on. This can be thought
    /// of as the message topic.
    #[inline]
    pub fn channel(&self) -> &str {
        self.0.channel.as_ref()
    }

    /// The payload of the notification. An empty payload is received as an
    /// empty string.
    #[inline]
    pub fn payload(&self) -> &str {
        self.0.payload.as_ref()
    }

    fn into_owned(self) -> PgNotification<'static> {
        PgNotification(self.0.into_owned())
    }
}

impl Debug for PgNotification<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgNotification")
            .field("process_id", &self.process_id())
            .field("channel", &self.channel())
            .field("payload", &self.payload())
            .finish()
    }
}

fn ident(mut name: &str) -> String {
    // If the input string contains a NUL byte, we should truncate the
    // identifier.
    if let Some(index) = name.find('\0') {
        name = &name[..index];
    }

    // Any double quotes must be escaped
    name.replace('"', "\"\"")
}

fn build_listen_all_query(channels: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    channels.into_iter().fold(String::new(), |mut acc, chan| {
        acc.push_str(r#"LISTEN ""#);
        acc.push_str(&ident(chan.as_ref()));
        acc.push_str(r#"";"#);
        acc
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_listen_all_query_with_single_channel() {
        let output = build_listen_all_query(&["test"]);
        assert_eq!(output.as_str(), r#"LISTEN "test";"#);
    }

    #[test]
    fn build_listen_all_query_with_multiple_channels() {
        let output = build_listen_all_query(&["channel.0", "channel.1"]);
        assert_eq!(output.as_str(), r#"LISTEN "channel.0";LISTEN "channel.1";"#);
    }
}
