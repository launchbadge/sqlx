use bytes::Bytes;
use failure::Error;
use super::super::{
    deserialize::{Deserialize, DeContext},
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
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let length = ctx.decoder.decode_length()?;
        let seq_no = ctx.decoder.decode_int_1();

        let column_packet = ColumnPacket::deserialize(ctx)?;

        let columns = if let Some(columns) = column_packet.columns {
            (0..columns)
                .map(|_| ColumnDefPacket::deserialize(ctx))
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<ColumnDefPacket>>()
        } else {
            Vec::new()
        };

        let mut rows = Vec::new();

        for _ in 0.. {
            // if end of buffer stop
            if ctx.decoder.eof() {
                break;
            }

            // Decode each column as string<lenenc>
            rows.push(
                (0..column_packet.columns.unwrap_or(0))
                    .map(|_| ctx.decoder.decode_string_lenenc())
                    .collect::<Vec<Bytes>>(),
            )
        }

        Ok(ResultSet {
            length,
            seq_no,
            column_packet,
            columns,
            rows,
        })
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use super::*;

    #[runtime::test]
    async fn it_decodes_result_set_packet() -> Result<(), Error> {
        let buf = Bytes::from(b"\
        \0\0\0\x01\
        \x02\0\0\x02\xff\x02
        \x01\0\0a\
        \x01\0\0b\
        \x01\0\0c\
        \x01\0\0d\
        \x01\0\0e\
        \x01\0\0f\
        \xfc\x01\x01\
        \x01\x01\
        \x01\x01\x01\x01\
        \x00\
        \x00\x00\
        \x01\
        \0\0\
        \x01\0\0g\
        \x01\0\0h\
        \x01\0\0i\
        \x01\0\0j\
        \x01\0\0k\
        \x01\0\0l\
        \xfc\x01\x01\
        \x01\x01\
        \x01\x01\x01\x01\
        \x00\
        \x00\x00\
        \x01\
        \0\0
        ".to_vec());
//        let message = ColumnDefPacket::deserialize(&mut Connection::mock().await, &mut Decoder::new(&buf))?;

        Ok(())
    }
}
