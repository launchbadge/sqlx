use bytes::Bytes;

#[derive(Debug, Default)]
pub struct ResultRow {
    pub columns: Vec<Option<Bytes>>
}

impl crate::mariadb::Deserialize for ResultRow {
    fn deserialize(ctx: &mut crate::mariadb::DeContext) -> Result<Self, failure::Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        let header = decoder.decode_int_u8();

        let bitmap = if let Some(columns) = ctx.columns {
            let size = (columns + 9) / 8;
            decoder.decode_byte_fix(size as usize)
        } else {
            Bytes::new()
        };

        let row = if let Some(columns) = ctx.columns {
            (0..columns).map(|index| {
                if (1 << index) & (bitmap[index/8] << (index % 8)) == 0 {
                    None
                } else {
                    match ctx.column_defs[index] {

                    }
                    decoder.decode_binary_column(&ctx.column_defs)
                }
            }).collect::<Vec<Bytes>>()
        } else {
            Vec::new()
        };

        Ok(ResultRow::default())
    }
}
