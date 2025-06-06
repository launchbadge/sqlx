use std::{
    collections::VecDeque,
    future::Future,
    ops::ControlFlow,
    pin::Pin,
    task::{ready, Context, Poll},
};

use crate::message::{
    BackendMessageFormat, FrontendMessage, Notification, ReadyForQuery, ReceivedMessage, Terminate,
};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::{SinkExt, StreamExt};
use sqlx_core::{
    bytes::Buf,
    net::{BufferedSocket, Socket},
    rt::spawn,
    Result,
};

use super::request::IoRequest;

#[derive(PartialEq, Debug)]
enum WorkerState {
    // The connection is open and ready for business.
    Open,
    // Sent/sending a [Terminate] message but did not close the socket. Responding to the last
    // messages but not receiving new ones.
    Closing,
    // The connection is terminated, this step closes the socket and stops the background task.
    Closed,
}

pub struct Worker {
    state: WorkerState,
    should_flush: bool,
    chan: UnboundedReceiver<IoRequest>,
    back_log: VecDeque<UnboundedSender<ReceivedMessage>>,
    socket: BufferedSocket<Box<dyn Socket>>,
    notif_chan: UnboundedSender<Notification>,
}

impl Worker {
    pub fn spawn(
        socket: BufferedSocket<Box<dyn Socket>>,
        notif_chan: UnboundedSender<Notification>,
    ) -> UnboundedSender<IoRequest> {
        let (tx, rx) = unbounded();

        let worker = Worker {
            state: WorkerState::Open,
            should_flush: false,
            chan: rx,
            back_log: VecDeque::new(),
            socket,
            notif_chan,
        };

        spawn(worker);
        tx
    }

    // Tries to receive the next message from the channel. Also handles termination if needed.
    #[inline(always)]
    fn poll_next_request(&mut self, cx: &mut Context<'_>) -> Poll<IoRequest> {
        if self.state != WorkerState::Open {
            return Poll::Pending;
        }

        match self.chan.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(request)) => Poll::Ready(request),
            Poll::Ready(None) => {
                // Channel was closed, explicitly or because the sender was dropped. Either way
                // we should start a gracefull shutdown.
                self.socket
                    .write_buffer_mut()
                    .put_slice(&[Terminate::FORMAT as u8, 0, 0, 0, 4]);

                self.state = WorkerState::Closing;
                self.should_flush = true;
                Poll::Pending
            }
        }
    }

    #[inline(always)]
    fn poll_receiver(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        if self.state != WorkerState::Open {
            return Poll::Ready(Ok(()));
        }

        loop {
            ready!(self.socket.poll_ready_unpin(cx))?;

            let request = ready!(self.poll_next_request(cx));

            self.socket.start_send_unpin(&request.data)?;
            self.should_flush = true;

            if let Some(chan) = request.chan {
                // We should send the responses back
                self.back_log.push_back(chan);
            } else {
            }
        }
    }

    #[inline(always)]
    fn handle_poll_flush(&mut self, cx: &mut Context<'_>) -> Result<()> {
        if self.should_flush && self.socket.poll_flush_unpin(cx).is_ready() {
            self.should_flush = false;
        }
        Ok(())
    }

    #[inline(always)]
    fn send_back(&mut self, response: ReceivedMessage) -> Result<()> {
        if let Some(chan) = self.back_log.front_mut() {
            let _ = chan.unbounded_send(response);
            Ok(())
        } else {
            Err(err_protocol!("Received response but did not expect one."))
        }
    }

    #[inline(always)]
    fn poll_backlog(&mut self, cx: &mut Context<'_>) -> Result<()> {
        while let Poll::Ready(response) = self.poll_next_message(cx)? {
            match response.format {
                BackendMessageFormat::ReadyForQuery => {
                    let rfq: ReadyForQuery = response.clone().decode()?;
                    self.send_back(response)?;
                    // Remove from the backlog so we dont send more responses back.
                    let _ = self.back_log.pop_front();
                }
                BackendMessageFormat::CopyInResponse => {
                    // End of response
                    self.send_back(response)?;
                    // Remove from the backlog so we dont send more responses back.
                    let _ = self.back_log.pop_front();
                }
                BackendMessageFormat::NotificationResponse => {
                    // Notification
                    let notif: Notification = response.decode()?;
                    let _ = self.notif_chan.unbounded_send(notif);
                }
                BackendMessageFormat::ParameterStatus => {
                    // Asynchronous response - todo
                }
                BackendMessageFormat::NoticeResponse => {
                    // Asynchronous response - todo
                }
                _ => self.send_back(response)?,
            }
        }

        if self.state != WorkerState::Open && self.back_log.is_empty() {
            self.state = WorkerState::Closed;
        }
        Ok(())
    }

    #[inline(always)]
    fn poll_next_message(&mut self, cx: &mut Context<'_>) -> Poll<Result<ReceivedMessage>> {
        self.socket.poll_try_read(cx, |buf| {
            // all packets in postgres start with a 5-byte header
            // this header contains the message type and the total length of the message
            let Some(mut header) = buf.get(..5) else {
                return Ok(ControlFlow::Continue(5));
            };

            let format = BackendMessageFormat::try_from_u8(header.get_u8())?;

            let message_len = header.get_u32() as usize;

            let expected_len = message_len
                .checked_add(1)
                // this shouldn't really happen but is mostly a sanity check
                .ok_or_else(|| err_protocol!("message_len + 1 overflows usize: {message_len}"))?;

            if buf.len() < expected_len {
                return Ok(ControlFlow::Continue(expected_len));
            }

            // `buf` SHOULD NOT be modified ABOVE this line

            // pop off the format code since it's not counted in `message_len`
            buf.advance(1);

            // consume the message, including the length prefix
            let mut contents = buf.split_to(message_len).freeze();

            // cut off the length prefix
            contents.advance(4);

            Ok(ControlFlow::Break(ReceivedMessage { format, contents }))
        })
    }

    #[inline(always)]
    fn poll_shutdown(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        if self.state == WorkerState::Closed {
            // The buffer is closed, a [Terminate] message has been sent, now try and close the socket.
            self.socket.poll_close_unpin(cx)
        } else {
            Poll::Pending
        }
    }
}

impl Future for Worker {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Try to receive responses from the database and handle them.
        self.poll_backlog(cx)?;

        // Push as many new requests in the write buffer as we can.
        if let Poll::Ready(Err(e)) = self.poll_receiver(cx) {
            return Poll::Ready(Err(e));
        };

        // Flush the write buffer if needed.
        self.handle_poll_flush(cx)?;

        // Close this socket if we're done.
        self.poll_shutdown(cx)
    }
}
