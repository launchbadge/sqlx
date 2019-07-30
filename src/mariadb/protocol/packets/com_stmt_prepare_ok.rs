use std::convert::TryFrom;

#[derive(Debug)]
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
            warnings
        })
    }
}
