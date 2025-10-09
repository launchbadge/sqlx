use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use bytes::{Buf, BytesMut};
use wasip3::sockets::types::{IpAddressFamily, IpSocketAddress, TcpSocket as WasiTcpSocket};
use wasip3::wit_stream;
use wasip3::wit_bindgen::StreamResult;

use crate::net::WithSocket;

mod socket;

// Modern WASI P3 JoinHandle using wit_bindgen's async primitives
pub struct JoinHandle<T> {
    future: Pin<Box<dyn Future<Output = T> + Send + 'static>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.future.as_mut().poll(cx)
    }
}

// we provide immediate execution with cooperative yielding for sqlx compatibility
pub fn spawn<T: 'static + Send>(fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T> {
    JoinHandle {
        future: Box::pin(async move {
            // Yield to allow other tasks to run cooperatively
            wasip3::wit_bindgen::yield_async().await;            
            fut.await
        }),
    }
}

// CPU-intensive operations using wit_bindgen's yield_blocking
pub fn spawn_blocking<F, R>(f: F) -> impl Future<Output = R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    async move {
        // Yield to allow other tasks to run before blocking operation
        wasip3::wit_bindgen::yield_blocking();
        f()
    }
}

// Native async yielding
pub async fn yield_now() {
    wasip3::wit_bindgen::yield_async().await
}

// Modern WASI P3 TcpSocket using wit_stream for async I/O
pub struct TcpSocket {
    wasi_socket: WasiTcpSocket,
    read_buffer: BytesMut,
}

impl TcpSocket {
    fn new(wasi_socket: WasiTcpSocket) -> Self {
        Self {
            wasi_socket,
            read_buffer: BytesMut::new(),
        }
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Serve from buffer first
        if !self.read_buffer.is_empty() {
            let to_copy = std::cmp::min(buf.len(), self.read_buffer.len());
            buf[..to_copy].copy_from_slice(&self.read_buffer[..to_copy]);
            self.read_buffer.advance(to_copy);
            return Ok(to_copy);
        }

        // Read from WASI socket stream
        let (mut stream, _fut) = self.wasi_socket.receive();
        match stream.read(Vec::with_capacity(buf.len())).await {
            (StreamResult::Complete(_), data) => {
                let to_copy = std::cmp::min(buf.len(), data.len());
                buf[..to_copy].copy_from_slice(&data[..to_copy]);
                
                // Buffer remaining data
                if data.len() > to_copy {
                    self.read_buffer.extend_from_slice(&data[to_copy..]);
                }
                
                Ok(to_copy)
            }
            (StreamResult::Dropped, _) => Ok(0),
            (StreamResult::Cancelled, _) => {
                Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Read cancelled"))
            }
        }
    }

    pub async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let (mut tx, rx) = wit_stream::new();
        
        // Start the send operation asynchronously
        let send_fut = self.wasi_socket.send(rx);
        
        // Write the data
        let remaining = tx.write_all(buf.to_vec()).await;
        drop(tx);
        
        // Wait for send to complete
        send_fut.await.map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, format!("Send failed: {:?}", e))
        })?;
        
        if remaining.is_empty() {
            Ok(buf.len())
        } else {
            Ok(buf.len() - remaining.len())
        }
    }
}

pub async fn connect_tcp<Ws: WithSocket>(
    host: &str,
    port: u16,
    with_socket: Ws,
) -> crate::Result<Ws::Output> {
    let addresses = wasip3::sockets::ip_name_lookup::resolve_addresses(host.to_string()).await
        .map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("DNS failed: {:?}", e))))?;
    
    let ip = addresses.into_iter().next()
        .ok_or_else(|| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "No addresses found")))?;
    
    let addr = match ip {
        wasip3::sockets::types::IpAddress::Ipv4(ipv4) => {
            IpSocketAddress::Ipv4(wasip3::sockets::types::Ipv4SocketAddress {
                address: ipv4,
                port,
            })
        }
        wasip3::sockets::types::IpAddress::Ipv6(ipv6) => {
            IpSocketAddress::Ipv6(wasip3::sockets::types::Ipv6SocketAddress {
                address: ipv6,
                port,
                flow_info: 0,
                scope_id: 0,
            })
        }
    };

    let wasi_socket = WasiTcpSocket::create(IpAddressFamily::Ipv4).map_err(|e| {
        crate::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to create socket: {:?}", e),
        ))
    })?;
    
    wasi_socket.connect(addr).await.map_err(|e| {
        crate::Error::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("failed to connect to {}:{}: {:?}", host, port, e),
        ))
    })?;

    let tcp_socket = TcpSocket::new(wasi_socket);

    Ok(with_socket.with_socket(tcp_socket).await)
}