use bytes::Bytes;
use failure::Error;

use super::super::{
    decode::Decoder,
    deserialize::Deserialize,
    packets::{column::ColumnPacket, column_def::ColumnDefPacket},
};

#[derive(Debug, Default)]
pub struct ResultSet {
    pub length: u32,
    pub seq_no: u8,
    pub column_packet: ColumnPacket,
    pub columns: Vec<ColumnDefPacket>,
    pub rows: Vec<Vec<Bytes>>,
}

impl Deserialize for ResultSet {
    fn deserialize(decoder: &mut Decoder) -> Result<Self, Error> {
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();

        let column_packet = ColumnPacket::deserialize(decoder)?;

        let columns = if let Some(columns) = column_packet.columns {
            (0..columns)
                .map(|_| ColumnDefPacket::deserialize(decoder))
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<ColumnDefPacket>>()
        } else {
            Vec::new()
        };

        let mut rows = Vec::new();

        for _ in 0.. {
            // if end of buffer stop
            if decoder.eof() {
                break;
            }

            // Decode each column as string<lenenc>
            rows.push(
                (0..column_packet.columns.unwrap_or(0))
                    .map(|_| decoder.decode_string_lenenc())
                    .collect::<Vec<Bytes>>(),
            )
        }

        Ok(ResultSet { length, seq_no, column_packet, columns, rows })
    }
}
