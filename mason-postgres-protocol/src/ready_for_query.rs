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
    fn size_hint(&self) -> usize { 6 }

    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.write_u8(b'Z')?;
        buf.write_u32::<BE>(5)?;
        buf.write_u8(self.status as u8)?;

        Ok(())
    }
}

impl Decode for ReadyForQuery {
    fn decode(b: Bytes) -> io::Result<Self> {
        if b.len() != 1 {
            return Err(io::ErrorKind::InvalidInput)?;
        }

        Ok(Self {
            status: match b[0] {
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
    use crate::{Decode, Encode, Message};
    use bytes::Bytes;
    use std::io;

    #[test]
    fn it_encodes_ready_for_query() -> io::Result<()> {
        let message = ReadyForQuery { status: TransactionStatus::Error };
        assert_eq!(&*message.to_bytes()?, &b"Z\0\0\0\x05E"[..]);

        Ok(())
    }

    #[test]
    fn it_decodes_ready_for_query() -> io::Result<()> {
        // FIXME: A test-utils type thing could be useful here as these 7 lines are quite..
        //        duplicated

        let b = Bytes::from_static(b"Z\0\0\0\x05E");
        let message = Message::decode(b)?;
        let body = if let Message::ReadyForQuery(body) = message {
            body
        } else {
            unreachable!();
        };

        assert_eq!(body.status, TransactionStatus::Error);

        Ok(())
    }
}
