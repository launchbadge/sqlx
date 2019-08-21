use crate::mariadb::{
    BufMut, ColumnDefPacket, ConnContext, Connection, Encode, FieldDetailFlag, FieldType,
    StmtExecFlag,
};
use bytes::Bytes;
use failure::Error;

#[derive(Debug)]
pub struct ComStmtExec {
    pub stmt_id: i32,
    pub flags: StmtExecFlag,
    pub params: Option<Vec<Option<Bytes>>>,
    pub param_defs: Option<Vec<ColumnDefPacket>>,
}

impl Encode for ComStmtExec {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::BinaryProtocol::ComStmtExec.into());
        buf.put_int_i32(self.stmt_id);
        buf.put_int_u8(self.flags.0);
        buf.put_int_u32(1);

        match (&self.params, &self.param_defs) {
            (Some(params), Some(param_defs)) if params.len() > 0 => {
                let null_bitmap_size = (params.len() + 7) / 8;
                let mut shift_amount = 0u8;
                let mut bitmap = vec![0u8];
                let send_type = 1u8;

                // Generate NULL-bitmap from params
                for param in params {
                    if param.is_none() {
                        let last_byte = bitmap.pop().unwrap();
                        bitmap.push(last_byte & (1 << shift_amount));
                    }

                    shift_amount = (shift_amount + 1) % 8;

                    if shift_amount % 8 == 0 {
                        bitmap.push(0u8);
                    }
                }

                buf.put_byte_fix(&Bytes::from(bitmap), null_bitmap_size);
                buf.put_int_u8(send_type);

                if send_type > 0 {
                    for param in param_defs {
                        buf.put_int_u8(param.field_type.0);
                        buf.put_int_u8(0);
                    }
                }

                // Encode params
                for index in 0..params.len() {
                    if let Some(bytes) = &params[index] {
                        buf.put_byte_lenenc(&bytes);
                    }
                }
            }
            _ => {}
        }

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{FieldDetailFlag, FieldType};

    #[test]
    fn it_encodes_com_stmt_close() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComStmtExec {
            stmt_id: 1,
            flags: StmtExecFlag::NoCursor,
            params: Some(vec![Some(Bytes::from_static(b"\x06daniel"))]),
            param_defs: Some(vec![ColumnDefPacket {
                catalog: Bytes::from_static(b"def"),
                schema: Bytes::from_static(b"test"),
                table_alias: Bytes::from_static(b"users"),
                table: Bytes::from_static(b"users"),
                column_alias: Bytes::from_static(b"username"),
                column: Bytes::from_static(b"username"),
                length_of_fixed_fields: Some(0x0Cu64),
                char_set: 1,
                max_columns: 1,
                field_type: FieldType::MYSQL_TYPE_STRING,
                field_details: FieldDetailFlag::NOT_NULL,
                decimals: 0,
            }]),
        }
        .encode(&mut buf, &mut ctx)?;

        Ok(())
    }
}
