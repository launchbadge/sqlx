use crate::mariadb::{ComStmtPrepareOk, ColumnDefPacket, Capabilities, EofPacket};

#[derive(Debug)]
pub struct ComStmtPrepareResp {
    pub ok: ComStmtPrepareOk,
    pub param_defs: Option<Vec<ColumnDefPacket>>,
    pub res_columns: Option<Vec<ColumnDefPacket>>,
}

//int<1> 0x00 COM_STMT_PREPARE_OK header
//int<4> statement id
//int<2> number of columns in the returned result set (or 0 if statement does not return result set)
//int<2> number of prepared statement parameters ('?' placeholders)
//string<1> -not used-
//int<2> number of warnings

impl crate::mariadb::Deserialize for ComStmtPrepareResp {
    fn deserialize(ctx: &mut crate::mariadb::DeContext) -> Result<Self, failure::Error> {
        let decoder = &mut ctx.decoder;
        let length = decoder.decode_length()?;

        let ok = ComStmtPrepareOk::deserialize(ctx)?;

        let param_defs = if ok.params > 0 {
            let param_defs = (0..ok.params).map(|_| ColumnDefPacket::deserialize(ctx))
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<ColumnDefPacket>>();

            if (ctx.conn.capabilities & Capabilities::CLIENT_DEPRECATE_EOF).is_empty() {
                EofPacket::deserialize(ctx)?;
            }

            Some(param_defs)
        } else {
            None
        };

        let res_columns = if ok.columns > 0 {
            let param_defs = (0..ok.columns).map(|_| ColumnDefPacket::deserialize(ctx))
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<ColumnDefPacket>>();

            if (ctx.conn.capabilities & Capabilities::CLIENT_DEPRECATE_EOF).is_empty() {
                EofPacket::deserialize(ctx)?;
            }

            Some(param_defs)
        } else {
            None
        };

        Ok(ComStmtPrepareResp {
            ok,
            param_defs,
            res_columns
        })
    }
}
