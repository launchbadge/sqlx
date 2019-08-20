use super::Decode;
use bytes::Bytes;
use memchr::memrchr;
use std::{io, str};

#[derive(Debug)]
pub struct CommandComplete {
    pub rows: u64,
}

impl CommandComplete {
    pub fn decode2(src: &[u8]) -> Self {
        // Attempt to parse the last word in the command tag as an integer
        // If it can't be parased, the tag is probably "CREATE TABLE" or something
        // and we should return 0 rows

        let rows_start = memrchr(b' ',src).unwrap_or(0);
        let rows = unsafe {
            str::from_utf8_unchecked(&src[(rows_start + 1)..(src.len() - 1)])
        };

        Self {
            rows: rows.parse().unwrap_or(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandComplete, Decode};
    use bytes::Bytes;
    use std::io;

    const COMMAND_COMPLETE_INSERT: &[u8] = b"INSERT 0 1\0";
    const COMMAND_COMPLETE_UPDATE: &[u8] = b"UPDATE 512\0";
    const COMMAND_COMPLETE_CREATE_TABLE: &[u8] = b"CREATE TABLE\0";
    const COMMAND_COMPLETE_BEGIN: &[u8] = b"BEGIN\0";

    #[test]
    fn it_decodes_command_complete_for_insert() -> io::Result<()> {
        let message = CommandComplete::decode2(COMMAND_COMPLETE_INSERT);

        assert_eq!(message.rows, 1);

        Ok(())
    }

    #[test]
    fn it_decodes_command_complete_for_update() -> io::Result<()> {
        let message = CommandComplete::decode2(COMMAND_COMPLETE_UPDATE);

        assert_eq!(message.rows, 512);

        Ok(())
    }

    #[test]
    fn it_decodes_command_complete_for_begin() -> io::Result<()> {
        let message = CommandComplete::decode2(COMMAND_COMPLETE_BEGIN);

        assert_eq!(message.rows, 0);

        Ok(())
    }

    #[test]
    fn it_decodes_command_complete_for_create_table() -> io::Result<()> {
        let message = CommandComplete::decode2(COMMAND_COMPLETE_CREATE_TABLE);

        assert_eq!(message.rows, 0);

        Ok(())
    }
}
