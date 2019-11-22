use super::Decode;
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
    fn decode(buf: &[u8]) -> crate::Result<Self> {
        Ok(Self {
            status: match buf[0] {
                b'I' => TransactionStatus::Idle,
                b'T' => TransactionStatus::Transaction,
                b'E' => TransactionStatus::Error,

                status => {
                    return Err(protocol_err!(
                            "received {:?} for TransactionStatus in ReadyForQuery",
                            status
                    ).into());
                }
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, ReadyForQuery, TransactionStatus};

    const READY_FOR_QUERY: &[u8] = b"E";

    #[test]
    fn it_decodes_ready_for_query() {
        let message = ReadyForQuery::decode(READY_FOR_QUERY).unwrap();

        assert_eq!(message.status, TransactionStatus::Error);
    }
}
