use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use crate::PgClientError;

#[derive(Debug)]
#[repr(u8)]
pub(crate) enum TransactionStatus {
    /// Not in a transaction block.
    Idle = b'I',

    /// In a transaction block.
    Transaction = b'T',

    /// In a _failed_ transaction block. Queries will be rejected until block is ended.
    Error = b'E',
}

#[derive(Debug)]
pub(crate) struct ReadyForQuery {
    pub(crate) transaction_status: TransactionStatus,
}

impl Deserialize<'_> for ReadyForQuery {
    fn deserialize_with(buf: Bytes, _: ()) -> Result<Self> {
        let status = match buf[0] {
            b'I' => TransactionStatus::Idle,
            b'T' => TransactionStatus::Transaction,
            b'E' => TransactionStatus::Error,

            status => {
                return Err(PgClientError::UnknownTransactionStatus(status).into());
            }
        };

        Ok(Self { transaction_status: status })
    }
}
