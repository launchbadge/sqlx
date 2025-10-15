use bytes::BytesMut;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::sync::Arc;
//use wasip3::sockets::types::IpSocketAddress;
use wasip3::wit_bindgen::rt::async_support;
use wasip3::wit_bindgen::rt::async_support::futures::channel::oneshot;

use crate::net::WithSocket;

mod socket;

pub struct JoinHandle<T> {
    rx: oneshot::Receiver<T>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(v)) => Poll::Ready(v),
            Poll::Ready(Err(oneshot::Canceled)) => panic!("wasip3 JoinHandle canceled"),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub fn spawn<T: 'static>(fut: impl Future<Output = T> + 'static) -> JoinHandle<T> {
    let (tx, rx) = oneshot::channel();
    async_support::spawn(async move {
        let v = fut.await;
        _ = tx.send(v);
    });
    JoinHandle { rx }
}

pub struct TcpSocket {
    pub tx: tokio_util::sync::PollSender<Vec<u8>>,
    pub rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    pub buf: BytesMut,
    pub task: tokio::task::JoinHandle<()>,
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        self.task.abort()
    }
}

pub async fn connect_tcp<Ws: WithSocket>(
    _host: &str,
    port: u16,
    with_socket: Ws,
) -> crate::Result<Ws::Output> {
    // address resolution requires additional processing
    // let addresses = wasip3::sockets::ip_name_lookup::resolve_addresses(host.to_string())
    //     .await
    //     .map_err(|e| {
    //         crate::Error::Io(std::io::Error::new(
    //             std::io::ErrorKind::Other,
    //             format!("DNS failed: {:?}", e),
    //         ))
    //     })?;

    // let ip = addresses.into_iter().next().ok_or_else(|| {
    //     crate::Error::Io(std::io::Error::new(
    //         std::io::ErrorKind::Other,
    //         "No addresses found",
    //     ))
    // })?;

    // let addr = match ip {
    //     wasip3::sockets::types::IpAddress::Ipv4(ipv4) => {
    //         IpSocketAddress::Ipv4(wasip3::sockets::types::Ipv4SocketAddress {
    //             address: ipv4,
    //             port,
    //         })
    //     }
    //     wasip3::sockets::types::IpAddress::Ipv6(ipv6) => {
    //         IpSocketAddress::Ipv6(wasip3::sockets::types::Ipv6SocketAddress {
    //             address: ipv6,
    //             port,
    //             flow_info: 0,
    //             scope_id: 0,
    //         })
    //     }
    // };
    let sock =
        wasip3::sockets::types::TcpSocket::create(wasip3::sockets::types::IpAddressFamily::Ipv4)
            .expect("failed to create TCP socket");
    sock.connect(wasip3::sockets::types::IpSocketAddress::Ipv4(
        wasip3::sockets::types::Ipv4SocketAddress {
            address: (127, 0, 0, 1),
            port,
        },
    ))
    .await
    .expect(&format!("failed to connect to 127.0.0.1:{port}"));

    // explicit channel item types so the compiler can infer types used below
    let (rx_tx, rx_rx): (
        tokio::sync::mpsc::Sender<Vec<u8>>,
        tokio::sync::mpsc::Receiver<Vec<u8>>,
    ) = tokio::sync::mpsc::channel(1);
    let (tx_tx, mut tx_rx): (
        tokio::sync::mpsc::Sender<Vec<u8>>,
        tokio::sync::mpsc::Receiver<Vec<u8>>,
    ) = tokio::sync::mpsc::channel(1);
    let (mut send_tx, send_rx) = wasip3::wit_stream::new();
    let (mut recv_rx, recv_fut) = sock.receive();

    let task = tokio::task::spawn_local(async move {
        let sock = Arc::new(sock);

        let (ready_tx, ready_rx) = oneshot::channel();
        async_support::spawn({
            let sock = Arc::clone(&sock);
            async move {
                let fut = sock.send(send_rx);
                _ = ready_tx.send(());
                _ = fut.await.unwrap();
                drop(sock);
            }
        });
        async_support::spawn({
            let sock = Arc::clone(&sock);
            async move {
                _ = recv_fut.await.unwrap();
                drop(sock);
            }
        });
        futures_util::join!(
            async {
                while let Some(result) = recv_rx.next().await {
                    _ = rx_tx.send(vec![result]).await;
                }
                drop(recv_rx);
                drop(rx_tx);
            },
            async {
                _ = ready_rx.await;
                while let Some(buf) = tx_rx.recv().await {
                    _ = send_tx.write(buf).await;
                }
                drop(tx_rx);
                drop(send_tx);
            },
        );
    });
    Ok(with_socket
        .with_socket(TcpSocket {
            tx: tokio_util::sync::PollSender::new(tx_tx),
            rx: rx_rx,
            buf: bytes::BytesMut::new(),
            task,
        })
        .await)
}
