use super::Decode;
use bytes::Bytes;
use memchr::memrchr;
use std::{io, str};

#[derive(Debug)]
pub struct CommandComplete {
    tag: Bytes,
}

impl CommandComplete {
    #[inline]
    pub fn tag(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.tag.as_ref()[..self.tag.len() - 1]) }
    }

    pub fn rows(&self) -> u64 {
        // Attempt to parse the last word in the command tag as an integer
        // If it can't be parased, the tag is probably "CREATE TABLE" or something
        // and we should return 0 rows

        let rows_start = memrchr(b' ', &*self.tag).unwrap_or(0);
        let rows_s = unsafe {
            str::from_utf8_unchecked(&self.tag.as_ref()[(rows_start + 1)..(self.tag.len() - 1)])
        };

        rows_s.parse().unwrap_or(0)
    }
}

impl Decode for CommandComplete {
    fn decode(src: Bytes) -> io::Result<Self> {
        Ok(Self { tag: src })
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
        let src = Bytes::from_static(COMMAND_COMPLETE_INSERT);
        let message = CommandComplete::decode(src)?;

        assert_eq!(message.tag(), "INSERT 0 1");
        assert_eq!(message.rows(), 1);

        Ok(())
    }

    #[test]
    fn it_decodes_command_complete_for_update() -> io::Result<()> {
        let src = Bytes::from_static(COMMAND_COMPLETE_UPDATE);
        let message = CommandComplete::decode(src)?;

        assert_eq!(message.tag(), "UPDATE 512");
        assert_eq!(message.rows(), 512);

        Ok(())
    }

    #[test]
    fn it_decodes_command_complete_for_begin() -> io::Result<()> {
        let src = Bytes::from_static(COMMAND_COMPLETE_BEGIN);
        let message = CommandComplete::decode(src)?;

        assert_eq!(message.tag(), "BEGIN");
        assert_eq!(message.rows(), 0);

        Ok(())
    }

    #[test]
    fn it_decodes_command_complete_for_create_table() -> io::Result<()> {
        let src = Bytes::from_static(COMMAND_COMPLETE_CREATE_TABLE);
        let message = CommandComplete::decode(src)?;

        assert_eq!(message.tag(), "CREATE TABLE");
        assert_eq!(message.rows(), 0);

        Ok(())
    }
}
