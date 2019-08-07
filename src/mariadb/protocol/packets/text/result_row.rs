use crate::mariadb::{DeContext, Decode, Decoder, ErrorCode, ServerStatusFlag};
use bytes::Bytes;
use failure::Error;
use std::convert::TryFrom;

#[derive(Default, Debug)]
pub struct ResultRow {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Vec<Option<Bytes>>,
}

impl Decode for ResultRow {
    fn decode(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        let columns = if let Some(columns) = ctx.columns {
            (0..columns)
                .map(|_| Some(decoder.decode_string_lenenc()))
                .collect::<Vec<Option<Bytes>>>()
        } else {
            Vec::new()
        };

        Ok(ResultRow {
            length,
            seq_no,
            columns,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        __bytes_builder,
        mariadb::{ConnContext, Decoder},
        ConnectOptions,
    };
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

        let _message = ResultRow::decode(&mut ctx)?;

        Ok(())
    }
}
