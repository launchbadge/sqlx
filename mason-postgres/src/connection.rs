use crate::protocol::{
    client::{PasswordMessage, Serialize, Terminate, StartupMessage},
    server::Message as ServerMessage,
};
use bytes::BytesMut;
use futures::{
    channel::mpsc,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    SinkExt, StreamExt,
};
use mason_core::ConnectOptions;
use md5::{Digest, Md5};
use runtime::{net::TcpStream, task::JoinHandle};
use std::{io};

pub struct Connection {
    writer: WriteHalf<TcpStream>,
    incoming: mpsc::UnboundedReceiver<ServerMessage>,

    // Buffer used when serializing outgoing messages
    wbuf: Vec<u8>,

    // Handle to coroutine reading messages from the stream
    receiver: JoinHandle<io::Result<()>>,

    // Process ID of the Backend
    process_id: i32,

    // Backend-unique key to use to send a cancel query message to the server
    secret_key: i32,
}

impl Connection {
    pub async fn establish(options: ConnectOptions<'_>) -> io::Result<Self> {
        let stream = TcpStream::connect((options.host, options.port)).await?;
        let (reader, writer) = stream.split();
        let (tx, rx) = mpsc::unbounded();
        let receiver = runtime::spawn(receiver(reader, tx));
        let mut conn = Self {
            wbuf: Vec::with_capacity(1024),
            writer,
            receiver,
            incoming: rx,
            process_id: -1,
            secret_key: -1,
        };

        // See this doc for more runtime parameters
        // https://www.postgresql.org/docs/12/runtime-config-client.html
        let params = [
            ("user", options.user),
            ("database", options.database),
            // TODO: Expose this property perhaps?
            (
                "application_name",
                Some(concat!(env!("CARGO_PKG_NAME"), "/v", env!("CARGO_PKG_VERSION"))),
            ),
            // Sets the display format for date and time values,
            // as well as the rules for interpreting ambiguous date input values.
            ("DateStyle", Some("ISO, MDY")),
            // Sets the display format for interval values.
            ("IntervalStyle", Some("iso_8601")),
            // Sets the time zone for displaying and interpreting time stamps.
            ("TimeZone", Some("UTC")),
            // Adjust postgres to return percise values for floats
            // NOTE: This is default in postgres 12+
            ("extra_float_digits", Some("3")),
            // Sets the client-side encoding (character set).
            ("client_encoding", Some("UTF-8")),
        ];

        conn.send(StartupMessage { params: &params }).await?;

        while let Some(message) = conn.incoming.next().await {
            match message {
                ServerMessage::AuthenticationOk => {
                    // Do nothing; server is just telling us that
                    // there is no password needed
                }

                ServerMessage::AuthenticationCleartextPassword => {
                    conn.send(PasswordMessage { password: options.password.unwrap_or_default() }).await?;
                }

                ServerMessage::AuthenticationMd5Password(body) => {
                    // Hash password|username
                    // FIXME: ConnectOptions should prepare a default user
                    let pass_user =
                        md5(options.password.unwrap_or_default(), options.user.unwrap_or_default());

                    let with_salt = md5(pass_user, body.salt());
                    let password = format!("md5{}", with_salt);

                    conn.send(PasswordMessage { password: &password }).await?;
                }

                ServerMessage::BackendKeyData(body) => {
                    conn.process_id = body.process_id();
                    conn.secret_key = body.secret_key();
                }

                ServerMessage::ReadyForQuery(_) => {
                    // Good to go
                    break;
                }

                _ => {
                    unimplemented!("received {:?} unimplemented message", message);
                }
            }
        }

        Ok(conn)
    }

    pub async fn close(mut self) -> io::Result<()> {
        self.send(Terminate).await?;
        self.writer.close().await?;
        self.receiver.await?;

        Ok(())
    }

    // Send client-serializable message to the server
    async fn send<S>(&mut self, message: S) -> io::Result<()>
    where
        S: Serialize,
    {
        self.wbuf.clear();

        message.serialize(&mut self.wbuf);

        self.writer.write_all(&self.wbuf).await?;
        self.writer.flush().await?;

        Ok(())
    }
}

async fn receiver(
    mut reader: ReadHalf<TcpStream>,
    mut sender: mpsc::UnboundedSender<ServerMessage>,
) -> io::Result<()> {
    let mut rbuf = BytesMut::with_capacity(0);
    let mut len = 0;

    loop {
        // This uses an adaptive system to extend the vector when it fills. We want to
        // avoid paying to allocate and zero a huge chunk of memory if the reader only
        // has 4 bytes while still making large reads if the reader does have a ton
        // of data to return.

        // See: https://github.com/rust-lang-nursery/futures-rs/blob/master/futures-util/src/io/read_to_end.rs#L50-L54

        if len == rbuf.len() {
            rbuf.reserve(32);

            unsafe {
                // Set length to the capacity and efficiently
                // zero-out the memory
                rbuf.set_len(rbuf.capacity());
                reader.initializer().initialize(&mut rbuf[len..]);
            }
        }

        // TODO: Need a select! on a channel that I can trigger to cancel this
        let cnt = reader.read(&mut rbuf[len..]).await?;

        if cnt > 0 {
            len += cnt;
        } else {
            // Read 0 bytes from the server; end-of-stream
            break;
        }

        while len > 0 {
            let size = rbuf.len();
            let message = ServerMessage::deserialize(&mut rbuf)?;
            len -= size - rbuf.len();

            // TODO: Some messages should be kept behind here
            match message {
                Some(ServerMessage::ParameterStatus(body)) => {
                    log::debug!("parameter {} = {}", body.name()?, body.value()?);
                }

                Some(message) => {
                    // TODO: Handle this error?
                    sender.send(message).await.unwrap();
                }

                None => {
                    // Did not receive enough bytes to
                    // deserialize a complete message
                    break;
                }
            }
        }
    }

    Ok(())
}

#[inline]
fn md5(a: impl AsRef<[u8]>, b: impl AsRef<[u8]>) -> String {
    hex::encode(Md5::new().chain(a).chain(b).result())
}
