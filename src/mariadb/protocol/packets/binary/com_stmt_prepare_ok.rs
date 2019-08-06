use std::convert::TryFrom;

#[derive(Debug, Default)]
pub struct ComStmtPrepareOk {
    pub stmt_id: i32,
    pub columns: i16,
    pub params: i16,
    pub warnings: i16,
}

impl crate::mariadb::Deserialize for ComStmtPrepareOk {
    fn deserialize(ctx: &mut crate::mariadb::DeContext) -> Result<Self, failure::Error> {
        let decoder = &mut ctx.decoder;
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        let header = decoder.decode_int_u8();

        let stmt_id = decoder.decode_int_i32();

        let columns = decoder.decode_int_i16();
        let params = decoder.decode_int_i16();

        // Skip 1 unused byte;
        decoder.skip_bytes(1);

        let warnings = decoder.decode_int_i16();

        Ok(ComStmtPrepareOk {
            stmt_id,
            columns,
            params,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        __bytes_builder,
        mariadb::{ConnContext, DeContext, Deserialize},
        ConnectOptions,
    };
    use bytes::Bytes;

    #[test]
    fn it_decodes_com_stmt_prepare_ok() -> Result<(), failure::Error> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<1> 0x00 COM_STMT_PREPARE_OK header
        0u8,
        // int<4> statement id
        1u8, 0u8, 0u8, 0u8,
        // int<2> number of columns in the returned result set (or 0 if statement does not return result set)
        10u8, 0u8,
        // int<2> number of prepared statement parameters ('?' placeholders)
        1u8, 0u8,
        // string<1> -not used-
        0u8,
        // int<2> number of warnings
        0u8, 0u8
        );

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, buf);

        let message = ComStmtPrepareOk::deserialize(&mut ctx)?;

        assert_eq!(message.stmt_id, 1);
        assert_eq!(message.columns, 10);
        assert_eq!(message.params, 1);
        assert_eq!(message.warnings, 0);

        Ok(())
    }
}
