use crate::protocol::{
    client::Serialize,
    server::Message as ServerMessage,
};
use bytes::BytesMut;
use futures::{
    channel::mpsc,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    SinkExt, StreamExt,
};
use mason_core::ConnectOptions;
use runtime::{net::TcpStream, task::JoinHandle};
use std::io;

mod establish;
// mod query;

pub struct Connection {
    writer: WriteHalf<TcpStream>,
    incoming: mpsc::UnboundedReceiver<ServerMessage>,

    // Buffer used when serializing outgoing messages
    wbuf: Vec<u8>,

    // Handle to coroutine reading messages from the stream
    receiver: JoinHandle<io::Result<()>>,

    // MariaDB Connection ID
    connection_id: i32,
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
            connection_id: -1,
        };

        establish::establish(&mut conn, options).await?;

        Ok(conn)
    }

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
        println!("Inside receiver loop");
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    Ok(())
}
