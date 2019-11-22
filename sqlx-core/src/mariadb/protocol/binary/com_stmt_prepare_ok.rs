use crate::io::Buf;
use byteorder::LittleEndian;
use std::io;

// https://mariadb.com/kb/en/library/com_stmt_prepare/#com_stmt_prepare_ok
#[derive(Debug)]
pub struct ComStmtPrepareOk {
    pub statement_id: u32,

    /// Number of columns in the returned result set (or 0 if statement does not return result set).
    pub columns: u16,

    /// Number of prepared statement parameters ('?' placeholders).
    pub params: u16,

    /// Number of warnings.
    pub warnings: u16,
}

impl ComStmtPrepareOk {
    pub(crate) fn decode(mut buf: &[u8]) -> io::Result<Self> {
        let header = buf.get_u8()?;

        if header != 0x00 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("expected COM_STMT_PREPARE_OK (0x00); received {}", header),
            ));
        }

        let statement_id = buf.get_u32::<LittleEndian>()?;
        let columns = buf.get_u16::<LittleEndian>()?;
        let params = buf.get_u16::<LittleEndian>()?;

        // Skip 1 unused byte
        // -not used- : string<1>
        buf.advance(1);

        let warnings = buf.get_u16::<LittleEndian>()?;

        Ok(Self {
            statement_id,
            columns,
            params,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::__bytes_builder;

    #[test]
    fn it_decodes_com_stmt_prepare_ok() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = &__bytes_builder!(
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

        let message = ComStmtPrepareOk::decode(&buf)?;

        assert_eq!(message.statement_id, 1);
        assert_eq!(message.columns, 10);
        assert_eq!(message.params, 1);
        assert_eq!(message.warnings, 0);

        Ok(())
    }
}
