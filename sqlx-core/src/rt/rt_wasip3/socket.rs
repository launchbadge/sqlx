use core::task::{Context, Poll};

use bytes::BufMut as _;
use std::io;
use tokio::sync::mpsc::error::TryRecvError;

use crate::io::ReadBuf;
use crate::net::Socket;

impl Socket for super::TcpSocket {
    fn try_read(&mut self, buf: &mut dyn ReadBuf) -> io::Result<usize> {
        let n = buf.remaining_mut();
        if !self.buf.is_empty() {
            if self.buf.len() >= n {
                buf.put_slice(&self.buf.split_to(n));
            } else {
                buf.put_slice(&self.buf);
                self.buf.clear();
            }
            return Ok(n);
        }
        match self.rx.try_recv() {
            Ok(rx_vec) => {
                eprintln!("wasip3 socket: try_read got {} bytes from rx", rx_vec.len());
                // make the item type explicit so methods like `len` and `split_off` are known
                let mut rx: Vec<u8> = rx_vec;
                if rx.len() < n {
                    buf.put_slice(&rx);
                    Ok(rx.len())
                } else {
                    let tail = rx.split_off(n);
                    buf.put_slice(&rx);
                    self.buf.extend_from_slice(&tail);
                    Ok(n)
                }
            }
            Err(TryRecvError::Empty) => {
                eprintln!("wasip3 socket: try_read would block (Empty)");
                Err(io::ErrorKind::WouldBlock.into())
            }
            Err(TryRecvError::Disconnected) => Ok(0),
        }
    }

    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let Some(tx) = self.tx.get_ref() else {
            return Err(io::ErrorKind::ConnectionReset.into());
        };
        let n = buf.len();
        match tx.try_send(buf.to_vec()) {
            Ok(()) => {
                eprintln!("wasip3 socket: try_write sent {} bytes", n);
                Ok(n)
            }
            Err(e) => {
                eprintln!("wasip3 socket: try_write failed: {:?}", e);
                Err(io::ErrorKind::WouldBlock.into())
            }
        }
    }

    fn poll_read_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(v)) => {
                self.buf.extend(v);
                Poll::Ready(Ok(()))
            }
            Poll::Ready(None) => Poll::Ready(Ok(())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_write_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.tx.poll_reserve(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(..)) => Poll::Ready(Err(io::ErrorKind::ConnectionReset.into())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(&mut self, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
