use crate::mariadb::{ComStmtPrepareOk, ColumnDefPacket, Capabilities, EofPacket};

#[derive(Debug, Default)]
pub struct ComStmtPrepareResp {
    pub ok: ComStmtPrepareOk,
    pub param_defs: Option<Vec<ColumnDefPacket>>,
    pub res_columns: Option<Vec<ColumnDefPacket>>,
}

impl crate::mariadb::Deserialize for ComStmtPrepareResp {
    fn deserialize(ctx: &mut crate::mariadb::DeContext) -> Result<Self, failure::Error> {
        let ok = ComStmtPrepareOk::deserialize(ctx)?;

        let param_defs = if ok.params > 0 {
            let param_defs = (0..ok.params).map(|_| ColumnDefPacket::deserialize(ctx))
                .filter(Result::is_ok)
                .map(Result::unwrap)
                .collect::<Vec<ColumnDefPacket>>();

            if !ctx.conn.capabilities.contains(Capabilities::CLIENT_DEPRECATE_EOF) {
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

            if !ctx.conn.capabilities.contains(Capabilities::CLIENT_DEPRECATE_EOF) {
                EofPacket::deserialize(ctx)?;
            }

            Some(param_defs)
        } else {
            None
        };

        Ok(ComStmtPrepareResp {
            ok,
            param_defs,
            res_columns,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{__bytes_builder, ConnectOptions, mariadb::{ConnContext, DeContext, Deserialize}};

    #[test]
    fn it_decodes_com_stmt_prepare_resp() -> Result<(), failure::Error> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // ---------------------------- //
        // Statement Prepared Ok Packet //
        // ---------------------------- //

        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<1> 0x00 COM_STMT_PREPARE_OK header
        0u8,
        // int<4> statement id
        1u8, 0u8, 0u8, 0u8,
        // int<2> number of columns in the returned result set (or 0 if statement does not return result set)
        1u8, 0u8,
        // int<2> number of prepared statement parameters ('?' placeholders)
        1u8, 0u8,
        // string<1> -not used-
        0u8,
        // int<2> number of warnings
        0u8, 0u8,

        // Param column definition

        // ------------------------ //
        // Column Definition packet //
        // ------------------------ //
        // int<3> length
        52u8, 0u8, 0u8,
        // int<1> seq_no
        3u8,
        // string<lenenc> catalog (always 'def')
        3u8, b"def",
        // string<lenenc> schema
        4u8, b"test",
        // string<lenenc> table alias
        5u8, b"users",
        // string<lenenc> table
        5u8, b"users",
        // string<lenenc> column alias
        8u8, b"username",
        // string<lenenc> column
        8u8, b"username",
        // int<lenenc> length of fixed fields (=0xC)
        0x0C_u8,
        // int<2> character set number
        8u8, 0u8,
        // int<4> max. column size
        0xFF_u8, 0xFF_u8, 0u8, 0u8,
        // int<1> Field types
        0xFC_u8,
        // int<2> Field detail flag
        0x11_u8, 0x10_u8,
        // int<1> decimals
        0u8,
        // int<2> - unused -
        0u8, 0u8,

        // Result column definitions

        // ------------------------ //
        // Column Definition packet //
        // ------------------------ //
        // int<3> length
        52u8, 0u8, 0u8,
        // int<1> seq_no
        3u8,
        // string<lenenc> catalog (always 'def')
        3u8, b"def",
        // string<lenenc> schema
        4u8, b"test",
        // string<lenenc> table alias
        5u8, b"users",
        // string<lenenc> table
        5u8, b"users",
        // string<lenenc> column alias
        8u8, b"username",
        // string<lenenc> column
        8u8, b"username",
        // int<lenenc> length of fixed fields (=0xC)
        0x0C_u8,
        // int<2> character set number
        8u8, 0u8,
        // int<4> max. column size
        0xFF_u8, 0xFF_u8, 0u8, 0u8,
        // int<1> Field types
        0xFC_u8,
        // int<2> Field detail flag
        0x11_u8, 0x10_u8,
        // int<1> decimals
        0u8,
        // int<2> - unused -
        0u8, 0u8
        );

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, &buf);

        let message = ComStmtPrepareResp::deserialize(&mut ctx)?;

        Ok(())
    }
}
