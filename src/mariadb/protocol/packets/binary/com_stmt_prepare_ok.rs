use byteorder::LittleEndian;
use std::io;
use crate::mariadb::Capabilities;
use crate::io::Buf;

#[derive(Debug, Default)]
pub struct ComStmtPrepareOk {
    pub stmt_id: i32,
    pub columns: u16,
    pub params: u16,
    pub warnings: u16,
}

impl crate::mariadb::Decode<'_> for ComStmtPrepareOk {
    fn decode(buf: &[u8], _: Capabilities) -> io::Result<Self> {
        let header = buf.get_u8();

        let stmt_id = buf.get_i32::<LittleEndian>()?;

        let columns = buf.get_u16::<LittleEndian>()?;
        let params = buf.get_u16::<LittleEndian>()?;

        // Skip 1 unused byte;
        buf.advance(1);

        let warnings = buf.get_u16::<LittleEndian>()?;

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
    use crate::{__bytes_builder};

    #[test]
    fn it_decodes_com_stmt_prepare_ok() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = &__bytes_builder!(
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
        )[..];

        let message = ComStmtPrepareOk::decode(&buf, Capabilities::CLIENT_PROTOCOL_41)?;

        assert_eq!(message.stmt_id, 1);
        assert_eq!(message.columns, 10);
        assert_eq!(message.params, 1);
        assert_eq!(message.warnings, 0);

        Ok(())
    }
}
