use crate::Decode;
use bytes::Bytes;
use memchr::{memchr, memrchr};
use std::{io, str};

#[derive(Debug)]
pub struct CommandComplete {
    tag: Bytes,
}

impl CommandComplete {
    pub fn tag(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.tag.as_ref()) }
    }

    pub fn rows(&self) -> u64 {
        let rows_start = memrchr(b' ', &*self.tag).map_or(0, |i| i + 1);
        let rows_s =
            unsafe { str::from_utf8_unchecked(&self.tag[rows_start..(self.tag.len() - 1)]) };

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
    use super::CommandComplete;
    use crate::Decode;
    use bytes::Bytes;
    use std::io;

    const COMMAND_COMPLETE_INSERT: &[u8] = b"INSERT 0 512\0";
    const COMMAND_COMPLETE_CREATE_TABLE: &[u8] = b"CREATE TABLE\0";

    #[test]
    fn it_decodes_command_complete_for_insert() -> io::Result<()> {
        let src = Bytes::from_static(COMMAND_COMPLETE_INSERT);
        let message = CommandComplete::decode(src)?;

        assert_eq!(message.tag(), "INSERT 0 512");
        assert_eq!(message.rows(), 512);

        Ok(())
    }

    #[test]
    fn it_decodes_command_complete_for_create_table() -> io::Result<()> {
        let src = Bytes::from_static(COMMAND_COMPLETE_INSERT);
        let message = CommandComplete::decode(src)?;

        assert_eq!(message.tag(), "CREATE TABLE");
        assert_eq!(message.rows(), 0);

        Ok(())
    }
}
