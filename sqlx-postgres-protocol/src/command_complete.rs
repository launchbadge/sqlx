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
        let tag_end = memchr(b' ', &*self.tag).unwrap();
        unsafe { str::from_utf8_unchecked(&self.tag[..tag_end]) }
    }

    pub fn rows(&self) -> u64 {
        let rows_start = memrchr(b' ', &*self.tag).unwrap();
        let rows_s =
            unsafe { str::from_utf8_unchecked(&self.tag[(rows_start + 1)..(self.tag.len() - 1)]) };

        rows_s.parse().unwrap()
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

    const COMMAND_COMPLETE: &[u8] = b"INSERT 0 512\0";

    #[test]
    fn it_decodes_command_complete() -> io::Result<()> {
        let src = Bytes::from_static(COMMAND_COMPLETE);
        let message = CommandComplete::decode(src)?;

        assert_eq!(message.tag(), "INSERT");
        assert_eq!(message.rows(), 512);

        Ok(())
    }
}
