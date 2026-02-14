mod socket;
pub mod tls;

pub use socket::{
    connect_tcp, connect_uds, BufferedSocket, BufferStats, Socket, SocketIntoBox, WithSocket,
    WriteBuffer,
};
