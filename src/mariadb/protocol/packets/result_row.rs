use crate::mariadb::{
    Decoder, DeContext, Deserialize, ErrorCode, ServerStatusFlag,
};
use bytes::Bytes;
use failure::Error;
use std::convert::TryFrom;

#[derive(Default, Debug)]
pub struct ResultRow {
    pub length: u32,
    pub seq_no: u8,
    pub row: Vec<Bytes>,
}

impl Deserialize for ResultRow {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        let row = if let Some(columns) = ctx.columns {
            (0..columns).map(|_| decoder.decode_string_lenenc()).collect::<Vec<Bytes>>()
        } else {
            Vec::new()
        };

        Ok(ResultRow { length, seq_no, row })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{__bytes_builder, ConnectOptions, mariadb::{ConnContext, Decoder}};
    use bytes::Bytes;

    #[test]
    fn it_decodes_result_row_packet() -> Result<(), Error> {
        #[rustfmt::skip]
            let buf = __bytes_builder!(
            // int<3> length
            1u8, 0u8, 0u8,
            // int<1> seq_no
            1u8,
            // string<lenenc> column data
            1u8, b"s"
        );

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, buf);

        ctx.columns = Some(1);

        let _message = ResultRow::deserialize(&mut ctx)?;

        Ok(())
    }
}
