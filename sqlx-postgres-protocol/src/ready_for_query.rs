use crate::{Decode, Encode};
use byteorder::{WriteBytesExt, BE};
use bytes::Bytes;
use std::io;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum TransactionStatus {
    /// Not in a transaction block.
    Idle = b'I',

    /// In a transaction block.
    Transaction = b'T',

    /// In a _failed_ transaction block. Queries will be rejected until block is ended.
    Error = b'E',
}

/// `ReadyForQuery` is sent whenever the backend is ready for a new query cycle.
#[derive(Debug)]
pub struct ReadyForQuery {
    pub status: TransactionStatus,
}

impl Encode for ReadyForQuery {
    #[inline]
    fn size_hint(&self) -> usize {
        6
    }

    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.write_u8(b'Z')?;
        buf.write_u32::<BE>(5)?;
        buf.write_u8(self.status as u8)?;

        Ok(())
    }
}

impl Decode for ReadyForQuery {
    fn decode(src: Bytes) -> io::Result<Self> {
        if src.len() != 1 {
            return Err(io::ErrorKind::InvalidInput)?;
        }

        Ok(Self {
            status: match src[0] {
                // FIXME: Variant value are duplicated with declaration
                b'I' => TransactionStatus::Idle,
                b'T' => TransactionStatus::Transaction,
                b'E' => TransactionStatus::Error,

                _ => unreachable!(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ReadyForQuery, TransactionStatus};
    use crate::{Decode, Encode};
    use bytes::Bytes;
    use std::io;

    const READY_FOR_QUERY: &[u8] = b"E";

    #[test]
    fn it_encodes_ready_for_query() -> io::Result<()> {
        let message = ReadyForQuery {
            status: TransactionStatus::Error,
        };

        let mut dst = Vec::with_capacity(message.size_hint());
        message.encode(&mut dst)?;

        assert_eq!(&dst[5..], READY_FOR_QUERY);

        Ok(())
    }

    #[test]
    fn it_decodes_ready_for_query() -> io::Result<()> {
        let src = Bytes::from_static(READY_FOR_QUERY);
        let message = ReadyForQuery::decode(src)?;

        assert_eq!(message.status, TransactionStatus::Error);

        Ok(())
    }
}
