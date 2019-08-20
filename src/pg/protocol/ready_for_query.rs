use super::Decode;
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
    status: TransactionStatus,
}

impl ReadyForQuery {
    #[inline]
    pub fn status(&self) -> TransactionStatus {
        self.status
    }
}

impl Decode for ReadyForQuery {
    fn decode(src: &[u8]) -> Self {
        Self {
            status: match src[0] {
                // FIXME: Variant value are duplicated with declaration
                b'I' => TransactionStatus::Idle,
                b'T' => TransactionStatus::Transaction,
                b'E' => TransactionStatus::Error,

                status => panic!("received {:?} for TransactionStatus", status),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, ReadyForQuery, TransactionStatus};
    use bytes::Bytes;

    const READY_FOR_QUERY: &[u8] = b"E";

    #[test]
    fn it_decodes_ready_for_query() {
        let message = ReadyForQuery::decode(READY_FOR_QUERY);

        assert_eq!(message.status, TransactionStatus::Error);
    }
}
