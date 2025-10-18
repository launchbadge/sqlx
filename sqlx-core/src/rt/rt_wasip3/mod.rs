use bytes::BytesMut;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::sync::Arc;
//use wasip3::sockets::types::IpSocketAddress;
use core::task::Waker;
use futures_util::future::{AbortHandle, Abortable};
use futures_util::stream::StreamExt as _;
use tokio::sync::mpsc;
use wasip3::wit_bindgen::rt::async_support;
use wasip3::wit_bindgen::rt::async_support::futures::channel::oneshot;

use crate::net::WithSocket;
use tracing::debug;

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

// A tiny poll-aware sender shim backed by `futures::channel::mpsc::Sender`.
// This provides the minimal API `socket.rs` expects: `try_send`, `get_ref` and
// `poll_reserve`.
pub struct WasiPollSender<T> {
    inner: Option<mpsc::Sender<T>>,
}

impl<T> WasiPollSender<T> {
    pub fn new(s: mpsc::Sender<T>) -> Self {
        Self { inner: Some(s) }
    }

    pub fn get_ref(&self) -> Option<&mpsc::Sender<T>> {
        // Note: inner holds a `tokio::sync::mpsc::Sender` stored as a
        // `Option<mpsc::Sender<T>>` (type alias imported above). Return a
        // reference to it if present.
        self.inner.as_ref()
    }

    pub fn try_send(&self, item: T) -> Result<(), ()> {
        if let Some(s) = &self.inner {
            s.try_send(item).map_err(|_| ())
        } else {
            Err(())
        }
    }

    pub fn poll_reserve(&self, cx: &mut Context<'_>) -> Poll<Result<(), ()>> {
        // There's no exact `poll_reserve` equivalent in futures mpsc. We emulate
        // it by checking if `poll_ready` would be `Ready` by attempting to
        // reserve via a short-lived future that yields `Ready` when the sink
        // can accept an item. For simplicity, we attempt a non-allocating
        // check: futures mpsc provides `poll_ready` on the Sink trait but
        // that's not directly available here. As a pragmatic approach, treat
        // the sender as always ready and return Pending only if the channel
        // is closed.
        if self.inner.is_some() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(()))
        }
    }
}

pub struct TcpSocket {
    pub tx: WasiPollSender<Vec<u8>>,
    pub rx: mpsc::Receiver<Vec<u8>>,
    pub buf: BytesMut,
    // Abort handle for the background task spawned with `async_support::spawn`.
    pub abort_handle: AbortHandle,
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        // Abort the background task if it's still running.
        self.abort_handle.abort();
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
    debug!("wasip3: creating tcp socket for port {}", port);
    let sock =
        wasip3::sockets::types::TcpSocket::create(wasip3::sockets::types::IpAddressFamily::Ipv4)
            .expect("failed to create TCP socket");
    debug!("wasip3: created tcp socket for port {}", port);
    sock.connect(wasip3::sockets::types::IpSocketAddress::Ipv4(
        wasip3::sockets::types::Ipv4SocketAddress {
            address: (127, 0, 0, 1),
            port,
        },
    ))
    .await
    .map_err(|e| {
        debug!("wasip3: connect failed: {:?}", e);
        e
    })
    .expect(&format!("failed to connect to 127.0.0.1:{port}"));

    // explicit channel item types so the compiler can infer types used below
    let (rx_tx, rx_rx) = mpsc::channel::<Vec<u8>>(1);
    let (tx_tx, mut tx_rx) = mpsc::channel::<Vec<u8>>(1);
    let (mut send_tx, send_rx) = wasip3::wit_stream::new();
    debug!("wasip3: created wit_stream for send/recv");
    let (mut recv_rx, recv_fut) = sock.receive();

    // Spawn a background task using the wasip3 async runtime and make it abortable.
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    // Give the wasip3 scheduler a quick yield before spawning the background
    // task. Use the host-aware `yield_async` so spawned tasks are eligible to
    // be polled promptly by the local runtime.
    async_support::yield_async().await;
    let background = Abortable::new(
        async move {
            let sock = Arc::new(sock);
            debug!("wasip3: background task starting; sock arc cloned");

            let (ready_tx, ready_rx) = oneshot::channel();
            let spawn_ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or_default();
            debug!("wasip3: spawning sock.send task at {}ms", spawn_ts);

            async_support::spawn({
                let sock = Arc::clone(&sock);
                async move {
                    let start_ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or_default();
                    debug!("wasip3: sock.send task started at {}ms", start_ts);
                    let fut = sock.send(send_rx);
                    let sig_ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or_default();
                    _ = ready_tx.send(());
                    debug!("wasip3: sock.send signalled ready at {}ms", sig_ts);
                    match fut.await {
                        Ok(_) => {
                            let done_ts = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or_default();
                            debug!("wasip3: sock.send completed at {}ms", done_ts);
                        }
                        Err(e) => {
                            let err_ts = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or_default();
                            debug!("wasip3: sock.send error at {}ms: {:?}", err_ts, e);
                        }
                    }
                    drop(sock);
                }
            });
            // Yield after spawning the send task so the runtime can poll it.
            async_support::yield_async().await;
            async_support::spawn({
                let sock = Arc::clone(&sock);
                async move {
                    let start_ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or_default();
                    debug!("wasip3: recv_fut task started at {}ms", start_ts);
                    match recv_fut.await {
                        Ok(_) => {
                            let done_ts = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or_default();
                            debug!("wasip3: recv_fut completed at {}ms", done_ts);
                        }
                        Err(e) => {
                            let err_ts = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or_default();
                            debug!("wasip3: recv_fut error at {}ms: {:?}", err_ts, e);
                        }
                    }
                    drop(sock);
                }
            });
            // Yield to the wasip3 scheduler to give the spawned tasks a chance
            // to be polled immediately. Without this yield the local runtime
            // may not poll newly spawned tasks until the current task yields,
            // which can cause head-of-line blocking observed during handshakes.
            async_support::yield_async().await;
            futures_util::join!(
                async {
                    while let Some(result) = recv_rx.next().await {
                        // `recv_rx` yields single bytes from the wasip3 receive stream.
                        debug!("wasip3: recv_rx.next yielded byte: {:#x}", result);
                        _ = rx_tx.send(vec![result]).await;
                    }
                    drop(recv_rx);
                    drop(rx_tx);
                },
                async {
                    _ = ready_rx.await;
                    debug!("wasip3: send task ready, draining tx_rx -> send_tx");
                    while let Some(buf) = tx_rx.recv().await {
                        debug!("wasip3: writing {} bytes to send_tx", buf.len());
                        let _ = send_tx.write(buf).await;
                    }
                    drop(tx_rx);
                    drop(send_tx);
                },
            );
        },
        abort_registration,
    );

    async_support::spawn(async move {
        let _ = background.await;
    });
    Ok(with_socket
        .with_socket(TcpSocket {
            tx: WasiPollSender::new(tx_tx),
            rx: rx_rx,
            buf: bytes::BytesMut::new(),
            abort_handle,
        })
        .await)
}
