//! TLS support for MSSQL connections.
//!
//! The TDS protocol expects the TLS handshake exchange to be wrapped within TDS pre-login
//! messages. This is complex to achieve because we do not control the TLS clients: both rustls
//! and native-tls are unaware of this, so we need to do the wrapping and unwrapping on our own.
//! To that end, this module implements a `TlsStreamWrapper` stream that can be interposed
//! between the TLS client libraries and the backing TCP stream, and that performs the
//! necessary mutations on the stream when the TLS handshake mode is enabled.

use crate::error::Error;
use crate::mssql::connection::stream::{MssqlStream, Shutdown};
use crate::mssql::protocol::packet::PacketType;
use crate::mssql::{MssqlConnectOptions, MssqlSslMode};
use sqlx_rt::{AsyncRead, AsyncWrite};
use std::io::{IoSlice, IoSliceMut};
use std::pin::Pin;
use std::task::{Context, Poll};

// Cheat sheet.
//
// TDS packet header:
// * u8: Pre-login packet type (0x12).
// * u8: Status (0x00 for incomplete packets, 0x01 for end of message).
// * u16: Packet length (including header, big-endian).
// * u16: Server PID (should be zero).
// * u8: Packet ID (can be zero).
// * u8: Window (must be zero).
//
// TLS packet header:
// * u8: Handshake record (0x16) or ChangeCipherSpec record (0x14).
// * u16: TLS version.
// * u16: Packet length (not including header, big-endian).

#[derive(Debug)]
/// TLS packet types we care about.
enum TlsPacketType {
    ChangeCipherSpec = 0x14,
    Handshake = 0x16,
}

impl TlsPacketType {
    /// Validates a raw TLS packet type and returns an enum value.
    pub fn get(value: u8) -> std::io::Result<Self> {
        match value {
            0x14 => Ok(Self::ChangeCipherSpec),
            0x16 => Ok(Self::Handshake),
            _ => Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unexpected TLS packet header type {}", value))),
        }
    }
}

/// A TCP stream that permits wrapping the TLS handshake over TDS messages when `tls_wrapping` is
/// set to true, and that delegates all other operations to an inner stream.
///
/// For reads, this expects incoming data to have well-formed TDS packet headers, as this must
/// interpret their length. The TLS payload is passed through as an opaque blob.
///
/// For writes, this expects incoming data to have well-formed TLS packet headers, as this must
/// interpret their length.
pub(crate) struct TlsStreamWrapper<S>
where
    S: AsyncRead + AsyncWrite + Shutdown + Unpin,
{
    /// The wrapped TCP stream.
    inner: S,

    /// Set to true when TLS-over-TDS wrapping is desired.
    tls_wrapping: bool,

    /// Bytes of the TDS header read so far. This accumulates read bytes until they represent a
    /// complete header, at which point this is cleared and the number of pending bytes to read to
    /// complete a full TDS packet is stored in `read_pending`.
    read_header_buf: Vec<u8>,

    /// Number of bytes pending to read before the TDS packet is complete.
    read_pending: u16,
}

/// "Removes" the byte at `buf[index]` by shifting the array contents to the left. `len` specifies
/// the length of valid data within `buf` and doesn't need to match the actual `buf` length.
///
/// This is O(n) on the buffer length, but given that we only use this a handful of times during
/// the TLS handshake, and the handshake happens only once per connection, it's not a big deal.
fn remove_index(buf: &mut [u8], len: usize, index: usize) {
    assert!(index < len);
    for i in index..len-1 {
        buf[i] = buf[i + 1];
    }

    // Poison the rest of the buffer. Not necessary but aids debugging if something goes wrong.
    for i in len-1..buf.len() {
        buf[i] = 0xff;
    }
}

impl<S> TlsStreamWrapper<S>
where
    S: AsyncRead + AsyncWrite + Shutdown + Unpin,
{
    pub(crate) fn wrap_tls(&mut self, b: bool) {
        log::debug!("mssql: TLS wrapping over TDS set to {}", b);
        self.tls_wrapping = b;
    }

    /// Takes a read from the TLS client library and removes TDS headers.
    fn process_read(&mut self, buf: &mut [u8], mut len: usize) -> std::io::Result<usize> {
        log::debug!("mssql read: Starting read of up to {} bytes", len);

        let mut i = 0;
        while i < len {
            if self.read_pending == 0 {
                assert!(self.read_header_buf.len() < 8);
                self.read_header_buf.push(buf[i]);
                remove_index(buf, len, i);
                len -= 1;

                if self.read_header_buf.len() == 8 {
                    log::debug!("mssql read: TDS header is {:?}", &self.read_header_buf[0..8]);
                    let tds_type = self.read_header_buf[0];
                    if tds_type != (PacketType::PreLogin as u8) {
                        return Err(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Unexpected TDS packet header type {}", tds_type)));
                    }
                    let tds_length = ((self.read_header_buf[2] as u16) << 8)
                        | (self.read_header_buf[3] as u16);
                    if tds_length <= 8 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Packet length {} in TDS header is too short", tds_length)));
                    }
                    log::debug!("mssql read: TDS type {}, length {}", tds_type, tds_length);
                    self.read_pending = tds_length - 8;
                    self.read_header_buf.clear();
                }
            } else {
                i += 1;
                self.read_pending -= 1;
            }
        }

        log::debug!("mssql read: Unwrapped {} bytes", len);
        Ok(len)
    }

    /// Takes a write from the TLS client library and wraps the first TLS packet in the write buffer
    /// into a TDS packet. Returns the wrapped packet and the length to return to the client.
    ///
    /// Assumes that the TLS client library writes TLS packets in full with a single write, which
    /// seems to be the case for all supported libraries.
    fn process_write(buf: &[u8]) -> std::io::Result<(Vec<u8>, usize)> {
        log::debug!("mssql write: Starting write of {} bytes", buf.len());

        if buf.len() < 5 {
            // We could handle this case by buffering this write and waiting for the TLS packet to
            // be complete.  Given that the TLS libraries we interact with don't need this, skip it
            // for simplicity.
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Write too short; truncated TLS header at {} bytes", buf.len())));
        }
        log::debug!("mssql write: TLS header is {:?}", &buf[0..5]);

        let tls_type = TlsPacketType::get(buf[0])?;
        let tls_payload_length = ((buf[3] as u16) << 8) | (buf[4] as u16);
        let tls_length = tls_payload_length as usize + 5;
        log::debug!("mssql write: TLS type {:?}, total length {}", tls_type, tls_length);

        if tls_length > (std::u16::MAX as usize - 8) {
            // We could handle this case by splitting the TLS payload into two TDS fragments, but
            // that would add significant complexity to this algorithm and the TLS libraries we
            // interact with don't seem to need this.
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Cannot handle TLS payload with size {} (too long)", tls_payload_length)));
        } else if buf.len() < tls_length {
            // We could handle this case by buffering this write and waiting for the TLS payload to
            // be complete, or we could write the partial writes as different TDS fragments.  Given
            // that the TLS libraries we interact with don't need this, skip it for simplicity.
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Cannot handle partial TLS write (got {} of {} bytes)",
                        buf.len(), tls_length)));
        } else if buf.len() > tls_length {
            // The buffer contains more data than just one TLS packet. It's likely that what follows
            // the TLS packet is another TLS packet, but we cannot know for sure. It's safer to just
            // handle the first packet and return a partial write. Furthermore, I did try to wrap
            // all packets in a single write (which OpenSSL does during the handshake) and the
            // connection would result in an error.
            log::debug!(
                "mssql write: Write buffer contains more than one TLS packet; handling one");
        }
        let tds_length = tls_length + 8;

        let mut wrapped_buf = Vec::with_capacity(tls_length + 8);

        // Build the TDS header.
        wrapped_buf.push(PacketType::PreLogin as u8);
        match tls_type {
            TlsPacketType::ChangeCipherSpec => {
                // The server doesn't like seeing this TLS packet on its own TDS message but is
                // happy enough to consume it as a separate TDS packet when combined with the next
                // wrapped TLS packet.
                wrapped_buf.push(0x00);
            }
            TlsPacketType::Handshake => wrapped_buf.push(0x01),
        }
        wrapped_buf.push(((tds_length >> 8) & 0xff) as u8);
        wrapped_buf.push((tds_length & 0xff) as u8);
        wrapped_buf.extend(&[0x00, 0x00, 0x00, 0x00]);
        log::debug!("mssql write: Wrapped TDS header is {:?}", &wrapped_buf[0..8]);

        // Attach the TLS payload (but not the whole buffer!).
        wrapped_buf.extend(&buf[0..tls_length]);

        log::debug!("mssql write: Wrapped buf if {} bytes, returned write length is {}",
            wrapped_buf.len(), tls_length);
        Ok((wrapped_buf, tls_length))
    }
}

impl<S> From<S> for TlsStreamWrapper<S>
where
    S: AsyncRead + AsyncWrite + Shutdown + Unpin,
{
    fn from(t: S) -> Self {
        Self {
            inner: t,
            tls_wrapping: false,
            read_header_buf: Vec::with_capacity(8),
            read_pending: 0,
        }
    }
}

impl<S> Shutdown for TlsStreamWrapper<S>
where
    S: AsyncRead + AsyncWrite + Shutdown + Unpin,
{
    fn shutdown(&mut self, how: std::net::Shutdown) -> std::io::Result<()> {
        self.inner.shutdown(how)
    }
}

impl<S> AsyncRead for TlsStreamWrapper<S>
where
    S: AsyncRead + AsyncWrite + Shutdown + Unpin,
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
        -> Poll<std::io::Result<usize>>
    {
        if self.tls_wrapping {
            loop {
                let poll = Pin::new(&mut self.inner).poll_read(cx, buf);
                match poll {
                    Poll::Ready(Ok(0)) => {
                        if !self.read_header_buf.is_empty() {
                            return Poll::Ready(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("TDS packet too short at {} bytes",
                                    self.read_header_buf.len()))));
                        }

                        if self.read_pending > 0 {
                            return Poll::Ready(Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("TLS payload missing {} bytes", self.read_pending))));
                        }

                        return poll;
                    }
                    Poll::Ready(Ok(size)) => {
                        let new_size = self.process_read(buf, size)?;
                        assert!(new_size <= size);
                        if new_size > 0 {
                            return Poll::Ready(Ok(new_size));
                        }
                        // Not enough data to build a wrapped TDS packet; continue to read more.
                    }
                    _ => return poll,
                }
            }
        } else {
            Pin::new(&mut self.inner).poll_read(cx, buf)
        }
    }

    fn poll_read_vectored(mut self: Pin<&mut Self>, cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>]) -> Poll<std::io::Result<usize>>
    {
        assert!(!self.tls_wrapping, "Unexpected vectored read during TLS handshake");
        Pin::new(&mut self.inner).poll_read_vectored(cx, bufs)
    }
}

impl<S> AsyncWrite for TlsStreamWrapper<S>
where
    S: AsyncRead + AsyncWrite + Shutdown + Unpin,
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8])
        -> Poll<std::io::Result<usize>>
    {
        if self.tls_wrapping {
            let (wrapped_buf, write_len) = TlsStreamWrapper::<S>::process_write(buf)?;
            let poll = Pin::new(&mut self.inner).poll_write(cx, wrapped_buf.as_slice());
            match poll {
                Poll::Ready(Ok(size)) => {
                    if size != wrapped_buf.len() {
                        return Poll::Ready(Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Don't know how to handle a partial write of {} bytes vs. {}",
                                size, wrapped_buf.len()))));
                    }
                    assert!(write_len == size - 8);
                    Poll::Ready(Ok(write_len))
                }
                Poll::Pending => {
                    return Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Don't know how to handle a pending write")));
                }
                _ => return poll,
            }
        } else {
            Pin::new(&mut self.inner).poll_write(cx, buf)
        }
    }

    fn poll_write_vectored(mut self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>])
        -> Poll<std::io::Result<usize>>
    {
        assert!(!self.tls_wrapping, "Unexpected vectored write during TLS handshake");
        Pin::new(&mut self.inner).poll_write_vectored(cx, bufs)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>>
    {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>>
    {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

pub(super) async fn maybe_upgrade(
    stream: &mut MssqlStream,
    options: &MssqlConnectOptions,
) -> Result<(), Error> {
    match options.ssl_mode {
        MssqlSslMode::Disabled => (),

        MssqlSslMode::Preferred => {
            upgrade(stream, options).await?;
        }

        _ => {
            if !upgrade(stream, options).await? {
                return Err(Error::Tls("server does not support TLS".into()));
            }
        }
    }

    Ok(())
}

async fn upgrade(stream: &mut MssqlStream, options: &MssqlConnectOptions) -> Result<bool, Error> {
    let accept_invalid_certs = !matches!(
        options.ssl_mode,
        MssqlSslMode::VerifyCa | MssqlSslMode::VerifyIdentity
    );
    let accept_invalid_host_names = !matches!(options.ssl_mode, MssqlSslMode::VerifyIdentity);

    stream.wrap_tls(true);
    let result = stream
        .upgrade(
            &options.host,
            accept_invalid_certs,
            accept_invalid_host_names,
            options.ssl_ca.as_ref(),
        )
        .await;
    stream.wrap_tls(false);
    result?;
    Ok(true)
}

#[cfg(test)]
mod testutils {
    use super::*;
    use std::collections::VecDeque;
    use sqlx_rt::{AsyncReadExt, AsyncWriteExt};

    /// A TCP stream that yields golden data on each read and that captures all writes for further
    /// inspection.
    #[derive(Default)]
    pub(crate) struct MockTcpStream {
        is_shutdown: bool,
        reads: VecDeque<VecDeque<u8>>,
        writes: Vec<Vec<u8>>,
    }

    impl MockTcpStream {
        /// Adds a new golden read to the mock stream.
        pub(crate) fn add_golden_read<B: Into<Vec<u8>>>(mut self, r: B) -> MockTcpStream {
            self.reads.push_back(VecDeque::from(r.into()));
            self
        }
    }

    impl Shutdown for MockTcpStream {
        fn shutdown(&mut self, _how: std::net::Shutdown) -> std::io::Result<()> {
            self.is_shutdown = true;
            Ok(())
        }
    }

    impl AsyncRead for MockTcpStream {
        fn poll_read(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &mut [u8])
        -> Poll<std::io::Result<usize>>
        {
            assert!(!self.is_shutdown, "Stream already shut down");
            match self.reads.pop_front() {
                Some(mut data) => {
                    if data.len() <= buf.len() {
                        for i in 0..data.len() {
                            buf[i] = data[i];
                        }
                        Poll::Ready(Ok(data.len()))
                    } else {
                        for i in 0..buf.len() {
                            buf[i] = data.pop_front().unwrap();
                        }
                        self.reads.push_front(data);
                        Poll::Ready(Ok(buf.len()))
                    }
                }
                None => Poll::Ready(Ok(0)),
            }
        }

        fn poll_read_vectored(self: Pin<&mut Self>, _cx: &mut Context<'_>,
            _bufs: &mut [IoSliceMut<'_>]) -> Poll<std::io::Result<usize>>
        {
            panic!("Not implemented");
        }
    }

    impl AsyncWrite for MockTcpStream {
        fn poll_write(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8])
            -> Poll<std::io::Result<usize>>
        {
            assert!(!self.is_shutdown, "Stream already shut down");
            self.writes.push(buf.to_vec());
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_write_vectored(self: Pin<&mut Self>, _cx: &mut Context<'_>, _bufs: &[IoSlice<'_>])
            -> Poll<std::io::Result<usize>>
        {
            panic!("Not implemented")
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>>
        {
            panic!("Not implemented")
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>>
        {
            panic!("Not implemented")
        }
    }

    /// Helper to validate interactions with a given `MockTcpStream` wrapped by a
    /// `TlsStreamWrapper`.
    #[must_use]
    pub(crate) struct Checker {
        stream: TlsStreamWrapper<MockTcpStream>,
    }

    impl From<MockTcpStream> for Checker {
        fn from(stream: MockTcpStream) -> Checker {
            Checker { stream: TlsStreamWrapper::from(stream) }
        }
    }

    impl Checker {
        /// Reads up to `buf_len` bytes from the stream wrapper and checks that the read yields
        /// the `exp_bytes`.
        pub(crate) fn check_read(mut self, exp_bytes: &[u8], buf_len: usize) -> Checker {
            let mut buf = Vec::with_capacity(buf_len);
            for _ in 0..buf_len {
                buf.push(0);
            }
            let n = sqlx_rt::block_on(self.stream.read(&mut buf)).unwrap();
            assert!(exp_bytes.len() == n, "Read shorter than expected ({} vs. {})",
                exp_bytes.len(), n);
            assert_eq!(exp_bytes, &buf[0..exp_bytes.len()]);
            self
        }

        /// Reads up to `buf_len` bytes from the stream wrapper and checks that the read fails
        /// with the `exp_error` message.
        pub(crate) fn check_read_error(mut self, exp_error: &str, buf_len: usize) -> Checker {
            let mut buf = Vec::with_capacity(buf_len);
            for _ in 0..buf_len {
                buf.push(0);
            }
            let e = sqlx_rt::block_on(self.stream.read(&mut buf)).unwrap_err();
            assert_eq!(exp_error, format!("{}", e));
            self
        }

        /// Writes `buf` to the stream wrapper and verifies that `exp_write` bytes were written to
        /// the backing stream.
        pub(crate) fn check_write<B: Into<Vec<u8>>>(mut self, exp_len: usize, exp_write: B, buf: &[u8])
            -> Checker
        {
            let before_writes = self.stream.inner.writes.len();
            let n = sqlx_rt::block_on(self.stream.write(buf)).unwrap();
            let after_writes = self.stream.inner.writes.len();
            assert_eq!(
                before_writes + 1, after_writes,
                "Logical write resulted in more than one physical write");
            assert_eq!(exp_write.into(), self.stream.inner.writes[after_writes - 1]);
            assert_eq!(exp_len, n);
            self
        }

        /// Writes `buf` to the stream wrapper and checks that the write fails with the `exp_error`
        /// message.
        pub(crate) fn check_write_error(mut self, exp_error: &str, buf: &[u8]) -> Checker {
            let before_writes = self.stream.inner.writes.len();
            let e = sqlx_rt::block_on(self.stream.write_all(buf)).unwrap_err();
            let after_writes = self.stream.inner.writes.len();
            assert_eq!(
                before_writes, after_writes,
                "Erroneous logical write resulted in a physical write");
            assert_eq!(exp_error, format!("{}", e));
            self
        }

        /// Toggles the TLS wrapping property of the backing stream.
        pub(crate) fn set_wrap_tls(mut self, b: bool) -> Checker {
            self.stream.wrap_tls(b);
            self
        }

        /// Consumes the checker and performs final validation.
        pub(crate) fn verify(mut self) {
            assert!(self.stream.inner.reads.is_empty(), "Not all golden data was consumed");

            if !self.stream.inner.is_shutdown {
                self.stream.shutdown(std::net::Shutdown::Both).unwrap();
                assert!(self.stream.inner.is_shutdown);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::testutils::*;

    #[test]
    fn test_remove_index() {
        let mut buf = [5];
        remove_index(&mut buf, 1, 0);
        assert_eq!([0xff], buf);

        let mut buf = [5, 1, 3];
        remove_index(&mut buf, 3, 2);
        assert_eq!([5, 1, 3], buf);

        let mut buf = [1, 2, 3, 4, 5, 6, 7, 8];
        remove_index(&mut buf, 8, 0);
        assert_eq!([2, 3, 4, 5, 6, 7, 8, 0xff], buf);
        remove_index(&mut buf, 7, 3);
        assert_eq!([2, 3, 4, 6, 7, 8, 0xff, 0xff], buf);
        remove_index(&mut buf, 6, 5);
        assert_eq!([2, 3, 4, 6, 7, 0xff, 0xff, 0xff], buf);
    }

    #[test]
    fn test_tlsstreamwrapper_read_passthrough_shorter_than_buffer() {
        let stream = MockTcpStream::default()
            .add_golden_read("shorter")
            .add_golden_read("than the")
            .add_golden_read("buffer");
        Checker::from(stream)
            .check_read("shorter".as_bytes(), 10)
            .check_read("than the".as_bytes(), 10)
            .check_read("buffer".as_bytes(), 10)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_passthrough_longer_than_buffer() {
        let stream = MockTcpStream::default()
            .add_golden_read("longer than the buffer");
        Checker::from(stream)
            .check_read("longer tha".as_bytes(), 10)
            .check_read("n the buff".as_bytes(), 10)
            .check_read("er".as_bytes(), 10)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_passthrough_to_tls_transitions() {
        let stream = MockTcpStream::default()
            .add_golden_read("Before TLS")
            .add_golden_read([
                0x12, 0x01, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x99, 0x98, 0x97, // TLS opaque payload.
            ])
            .add_golden_read("After TLS");
        Checker::from(stream)
            .check_read("Before TLS".as_bytes(), 100)
            .set_wrap_tls(true)
            .check_read(&[0x99, 0x98, 0x97], 100)
            .set_wrap_tls(false)
            .check_read("After TLS".as_bytes(), 100)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_packets_fit_in_buffer() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x12, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x03, // TLS opaque payload.
            ])
            .add_golden_read([
                0x12, 0x01, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x41, 0x35, 0x98, 0x45, 0x19, // TLS opaque payload.
            ]);
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read(&[0x03], 100)
            .check_read(&[0x41, 0x35, 0x98, 0x45, 0x19], 100)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_one_by_one() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x12, 0x01, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x50, 0x51, // TLS opaque payload.
                0x12, 0x01, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x52, // TLS opaque payload.
            ]);

        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read(&[0x50], 1)
            .check_read(&[0x51], 1)
            .check_read(&[0x52], 1)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_packet_size_exceeds_one_byte() {
        let mut golden_read = vec![0x12, 0x01, 0x1b, 0x60, 0x00, 0x00, 0x00, 0x00]; // TDS header.
        let mut exp_read = vec![];
        // Generate a TLS payload that's long enough to require more than one byte to encode its
        // size in the TDS header.
        for i in 0..7000 {
            golden_read.push((i % 256) as u8);
            exp_read.push((i % 256) as u8);
        }

        let stream = MockTcpStream::default()
            .add_golden_read(golden_read);
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read(&exp_read, 8192)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_buffer_shorter_than_payload() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x12, 0x01, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x99, 0x98, 0x97, // TLS opaque payload.
            ]);

        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read(&[0x99, 0x98], 10)
            .check_read(&[0x97], 10)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_buffer_shorter_than_header() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x12, 0x01, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x99, 0x98, 0x97, // TLS opaque payload.
            ]);

        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read(&[0x99, 0x98], 5)
            .check_read(&[0x97], 5)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_invalid_header_type() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x14, 0x01, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x01, // TLS opaque payload.
            ]);

        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read_error("Unexpected TDS packet header type 20", 100)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_truncated_header() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x12, 0x01, 0x00, // TDS header.
            ]);

        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read_error("TDS packet too short at 3 bytes", 100)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_empty_payload() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x12, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x01, // TLS opaque payload.
            ]);

        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read_error("Packet length 8 in TDS header is too short", 100)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_invalid_tds_length() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x12, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x01, // TLS opaque payload.
            ]);

        Checker::from(stream)
            .set_wrap_tls(true)
            .check_read_error("Packet length 5 in TDS header is too short", 100)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_read_tls_truncated_payload() {
        let stream = MockTcpStream::default()
            .add_golden_read([
                0x12, 0x01, 0x01, 0x23, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x01, 0x02, // TLS opaque payload.
            ]);

        Checker::from(stream)
        .set_wrap_tls(true)
        .check_read(&[0x01, 0x02], 100)
        .check_read_error("TLS payload missing 281 bytes", 100)
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_passthrough() {
        let stream = MockTcpStream::default();
        Checker::from(stream)
            .check_write(3, "abc", "abc".as_bytes())
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_passthrough_to_tls_transitions() {
        let stream = MockTcpStream::default();
        Checker::from(stream)
            .check_write(10, "Before TLS", "Before TLS".as_bytes())
            .set_wrap_tls(true)
            .check_write(7, [
                0x12, 0x01, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x16, 0x00, 0x00, 0x00, 0x02, // TLS header.
                0x90, 0x91, // TLS payload.
            ], &[
                0x16, 0x00, 0x00, 0x00, 0x02, // TLS header.
                0x90, 0x91, // TLS payload.
            ])
            .set_wrap_tls(false)
            .check_write(9, "After TLS", "After TLS".as_bytes())
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_tls_handshake() {
        let stream = MockTcpStream::default();
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_write(6, [
                0x12, 0x01, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x16, 0x00, 0x00, 0x00, 0x01, // TLS header.
                0x01, // TLS payload.
            ], &[
                0x16, 0x00, 0x00, 0x00, 0x01, // TLS header.
                0x01, // TLS payload.
            ])
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_tls_changecipherspec() {
        let stream = MockTcpStream::default();
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_write(6, [
                0x12, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x14, 0x00, 0x00, 0x00, 0x01, // TLS header.
                0x01, // TLS payload.
            ], &[
                0x14, 0x00, 0x00, 0x00, 0x01, // TLS header.
                0x01, // TLS payload.
            ])
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_tls_payload_longer_than_tls_length() {
        let stream = MockTcpStream::default();
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_write(7, [
                0x12, 0x01, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, // TDS header.
                0x16, 0x00, 0x00, 0x00, 0x02, // TLS header.
                0x90, 0x91, // TLS payload.
            ], &[
                0x16, 0x00, 0x00, 0x00, 0x02, // TLS header.
                0x90, 0x91, // TLS payload.
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Trailing data ignored in this write.
            ])
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_tls_too_short() {
        let stream = MockTcpStream::default();
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_write_error("Write too short; truncated TLS header at 4 bytes", &[
                0x16, 0x00, 0x00, 0x00, // TLS header.
            ])
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_tls_payload_max_length() {
        let mut exp_buf = vec!();
        let mut buf = vec!();
        exp_buf.extend(&[0x12, 0x01, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00]);
        exp_buf.extend(&[0x16, 0x00, 0x00, 0xff, 0xf2]);
        buf.extend(&[0x16, 0x00, 0x00, 0xff, 0xf2]);
        for i in 0..0xfff2 {
            exp_buf.push((i % 256) as u8);
            buf.push((i % 256) as u8);
        }

        let stream = MockTcpStream::default();
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_write(buf.len(), exp_buf, buf.as_slice())
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_tls_payload_too_long() {
        let stream = MockTcpStream::default();
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_write_error("Cannot handle TLS payload with size 65523 (too long)", &[
                0x16, 0x00, 0x00, 0xff, 0xf3, // TLS header.
            ])
            .verify();
    }

    #[test]
    fn test_tlsstreamwrapper_write_tls_payload_shorter_than_tls_length() {
        let stream = MockTcpStream::default();
        Checker::from(stream)
            .set_wrap_tls(true)
            .check_write_error("Cannot handle partial TLS write (got 15 of 16 bytes)", &[
                0x16, 0x00, 0x00, 0x00, 0x0b, // TLS header.
                0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, // TLS payload.
            ])
            .verify();
    }
}
