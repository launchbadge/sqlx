use crate::mariadb::{StmtExecFlag, ColumnDefPacket, FieldDetailFlag};
use bytes::Bytes;

#[derive(Debug)]
pub struct ComStmtExec {
    pub stmt_id: i32,
    pub flags: StmtExecFlag,
    pub params: Option<Vec<Option<Bytes>>>,
    pub param_defs: Option<Vec<ColumnDefPacket>>,
}

impl crate::mariadb::Serialize for ComStmtExec {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), failure::Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(crate::mariadb::BinaryProtocol::ComStmtExec.into());
        encoder.encode_int_i32(self.stmt_id);
        encoder.encode_int_u8(self.flags as u8);
        encoder.encode_int_u8(0);

        if let Some(params) = &self.params {
            let null_bitmap_size = (params.len() + 7) / 8;
            let mut shift_amount = 0u8;
            let mut bitmap = vec![0u8];

            // Generate NULL-bitmap from params
            for param in params {
               if param.is_none() {
                   bitmap.push(bitmap.last().unwrap() & (1 << shift_amount));
                }

                shift_amount = (shift_amount + 1) % 8;

                if shift_amount % 8 == 0 {
                    bitmap.push(0u8);
                }
            }

            // Do not send the param types
            encoder.encode_int_u8(if self.param_defs.is_some() {
                1u8
            } else {
                0u8
            });

            if let Some(params) = &self.param_defs {
                for param in params {
                    encoder.encode_int_u8(param.field_type as u8);
                    encoder.encode_int_u8(if (param.field_details & FieldDetailFlag::UNSIGNED).is_empty() {
                        1u8
                    } else {
                        0u8
                    });
                }
            }

            // Encode params
            for param in params {

            }
        }

        encoder.encode_length();

        Ok(())
    }
}
