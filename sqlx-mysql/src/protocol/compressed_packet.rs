use crate::error::Error;
use crate::io::ProtocolEncode;
use crate::options::Compression;
use crate::CompressionConfig;
use bytes::{BufMut, Bytes};
use flate2::read::ZlibDecoder;
use flate2::{write::ZlibEncoder, Compression as ZlibCompression};
use sqlx_core::io::ProtocolDecode;
use std::io::{Cursor, Read, Write};

#[derive(Debug)]
pub(crate) struct CompressedPacket<T>(pub(crate) T);

pub(crate) struct CompressedPacketContext<'cs, C> {
    pub(crate) nested_context: C,
    pub(crate) sequence_id: &'cs mut u8,
    pub(crate) compression: CompressionConfig,
}

impl<'en, 'compressed_stream, T, C>
    ProtocolEncode<'en, CompressedPacketContext<'compressed_stream, C>> for CompressedPacket<T>
where
    T: ProtocolEncode<'en, C>,
{
    fn encode_with(
        &self,
        buf: &mut Vec<u8>,
        context: CompressedPacketContext<'compressed_stream, C>,
    ) -> Result<(), Error> {
        let mut uncompressed_payload = Vec::with_capacity(0xFF_FF_FF);
        self.0
            .encode_with(&mut uncompressed_payload, context.nested_context)?;

        let mut chunks = uncompressed_payload.chunks(0xFF_FF_FF);
        for chunk in chunks.by_ref() {
            add_packet(buf, *context.sequence_id, &context.compression, chunk)?;
            *context.sequence_id = context.sequence_id.wrapping_add(1);
        }

        Ok(())
    }
}

fn add_packet(
    buf: &mut Vec<u8>,
    sequence_id: u8,
    compression: &CompressionConfig,
    uncompressed_chunk: &[u8],
) -> Result<(), Error> {
    let offset = buf.len();
    buf.extend_from_slice(&[0; 7]);

    let compressed_payload_length = compress(compression, uncompressed_chunk, buf)?;

    let mut header = Vec::with_capacity(7);
    header.put_uint_le(compressed_payload_length as u64, 3);
    header.put_u8(sequence_id);
    header.put_uint_le(uncompressed_chunk.len() as u64, 3);
    buf[offset..offset + 7].copy_from_slice(&header);

    Ok(())
}

impl<'compressed_stream, C> ProtocolDecode<'_, CompressedPacketContext<'compressed_stream, C>>
    for CompressedPacket<Bytes>
{
    fn decode_with(
        buf: Bytes,
        context: CompressedPacketContext<'compressed_stream, C>,
    ) -> Result<Self, Error> {
        decompress(&context.compression, buf.as_ref()).map(|d| CompressedPacket(Bytes::from(d)))
    }
}

fn compress(
    compression: &CompressionConfig,
    input: &[u8],
    output: &mut Vec<u8>,
) -> Result<usize, Error> {
    let offset = output.len();
    let mut cursor = Cursor::new(output);
    cursor.set_position(offset as u64);

    let cursor = match compression {
        CompressionConfig(Compression::Zlib, level) => {
            let mut encoder = ZlibEncoder::new(cursor, ZlibCompression::new(*level as u32));
            let _ = encoder.write(input)?;
            encoder.finish()?
        }
        CompressionConfig(Compression::Zstd, level) => {
            zstd::stream::copy_encode(input, &mut cursor, *level as i32)?;
            cursor
        }
    };

    Ok(cursor.get_ref().len().saturating_sub(offset))
}

fn decompress(compression: &CompressionConfig, bytes: &[u8]) -> Result<Vec<u8>, Error> {
    match compression.0 {
        Compression::Zlib => {
            let mut out = Vec::with_capacity(bytes.len() * 2);
            ZlibDecoder::new(bytes).read_to_end(&mut out)?;
            Ok(out)
        }
        Compression::Zstd => Ok(zstd::stream::decode_all(bytes)?),
    }
}
