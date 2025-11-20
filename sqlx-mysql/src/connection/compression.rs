use crate::protocol::{Capabilities, CompressedPacket, CompressedPacketContext};
use crate::CompressionConfig;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use sqlx_core::io::{ProtocolDecode, ProtocolEncode};
use sqlx_core::net::{BufferedSocket, Socket};
use sqlx_core::Error;
use std::cmp::min;

pub(crate) struct CompressionMySqlStream<S = Box<dyn Socket>> {
    stream: CompressionStream,
    pub(crate) socket: BufferedSocket<S>,
}

impl<S: Socket> CompressionMySqlStream<S> {
    pub(crate) fn not_compressed(socket: BufferedSocket<S>) -> Self {
        let stream = CompressionStream::NotCompressed(NoCompressionStream {});
        Self { stream, socket }
    }

    pub(crate) fn compressed(socket: BufferedSocket<S>, compression: CompressionConfig) -> Self {
        let stream = CompressionStream::Compressed(CompressedStream::new(compression));
        Self { stream, socket }
    }

    pub(crate) fn boxed(self) -> CompressionMySqlStream<Box<dyn Socket>> {
        CompressionMySqlStream {
            socket: self.socket.boxed(),
            stream: self.stream,
        }
    }

    pub async fn read_with<'de, T, C>(&mut self, byte_len: usize, context: C) -> Result<T, Error>
    where
        T: ProtocolDecode<'de, C>,
    {
        match self.stream {
            CompressionStream::NotCompressed(ref mut s) => {
                s.read_with(byte_len, context, &mut self.socket).await
            }
            CompressionStream::Compressed(ref mut s) => {
                s.read_with(byte_len, context, &mut self.socket).await
            }
        }
    }

    pub fn write_with<'en, 'stream, T>(
        &mut self,
        value: T,
        context: (Capabilities, &'stream mut u8),
    ) -> Result<(), Error>
    where
        T: ProtocolEncode<'en, (Capabilities, &'stream mut u8)>,
    {
        match self.stream {
            CompressionStream::NotCompressed(ref mut s) => {
                s.write_with(value, context, &mut self.socket)
            }
            CompressionStream::Compressed(ref mut s) => {
                s.write_with(value, context, &mut self.socket)
            }
        }
    }
}

enum CompressionStream {
    NotCompressed(NoCompressionStream),
    Compressed(CompressedStream),
}

struct NoCompressionStream {}
impl NoCompressionStream {
    pub async fn read_with<'de, T, C, S: Socket>(
        &mut self,
        byte_len: usize,
        context: C,
        buffered_socket: &mut BufferedSocket<S>,
    ) -> Result<T, Error>
    where
        T: ProtocolDecode<'de, C>,
    {
        buffered_socket.read_with(byte_len, context).await
    }

    pub fn write_with<'en, 'stream, T, C, S: Socket>(
        &mut self,
        packet: T,
        context: C,
        buffered_socket: &mut BufferedSocket<S>,
    ) -> Result<(), Error>
    where
        T: ProtocolEncode<'en, C>,
    {
        buffered_socket.write_with(packet, context)
    }
}

struct CompressedStream {
    compression: CompressionConfig,
    sequence_id: u8,
    last_read_packet: Option<Bytes>,
}

impl CompressedStream {
    fn new(compression: CompressionConfig) -> Self {
        Self {
            sequence_id: 0,
            last_read_packet: None,
            compression,
        }
    }

    async fn receive_packet<S: Socket>(
        &mut self,
        buffered_socket: &mut BufferedSocket<S>,
    ) -> Result<Bytes, Error> {
        let mut header: Bytes = buffered_socket.read(7).await?;
        #[allow(clippy::cast_possible_truncation)]
        let compressed_payload_length = header.get_uint_le(3) as usize;
        let sequence_id = header.get_u8();
        let uncompressed_payload_length = header.get_uint_le(3);

        self.sequence_id = sequence_id.wrapping_add(1);

        let packet = if uncompressed_payload_length > 0 {
            let compressed_context = CompressedPacketContext {
                nested_context: (),
                sequence_id: &mut self.sequence_id,
                compression: self.compression,
            };
            let compressed_payload: CompressedPacket<Bytes> = buffered_socket
                .read_with(compressed_payload_length, compressed_context)
                .await?;

            compressed_payload.0
        } else {
            let uncompressed_payload: Bytes = buffered_socket
                .read_with(compressed_payload_length, ())
                .await?;

            uncompressed_payload
        };

        Ok(packet)
    }

    pub async fn read_with<'de, T, C, S: Socket>(
        &mut self,
        byte_len: usize,
        context: C,
        buffered_socket: &mut BufferedSocket<S>,
    ) -> Result<T, Error>
    where
        T: ProtocolDecode<'de, C>,
    {
        let mut result_buffer = BytesMut::with_capacity(byte_len);
        while result_buffer.len() != byte_len {
            let current_packet = match self.last_read_packet.as_mut() {
                None => {
                    let received_packet = self.receive_packet(buffered_socket).await?;
                    self.last_read_packet = Some(received_packet);
                    self.last_read_packet.as_mut().unwrap()
                }
                Some(p) => p,
            };

            let remaining_bytes_count = byte_len.saturating_sub(result_buffer.len());
            let available_bytes_count = min(current_packet.len(), remaining_bytes_count);
            let chunk = current_packet.split_to(available_bytes_count);
            result_buffer.put_slice(chunk.chunk());

            if current_packet.is_empty() {
                self.last_read_packet = None
            }
        }

        T::decode_with(result_buffer.freeze(), context)
    }

    pub fn write_with<'en, T, C, S: Socket>(
        &mut self,
        packet: T,
        context: C,
        buffered_socket: &mut BufferedSocket<S>,
    ) -> Result<(), Error>
    where
        T: ProtocolEncode<'en, C>,
    {
        self.sequence_id = 0;
        let compressed_packet = CompressedPacket(packet);
        buffered_socket.write_with(
            compressed_packet,
            CompressedPacketContext {
                nested_context: context,
                sequence_id: &mut self.sequence_id,
                compression: self.compression,
            },
        )
    }
}
