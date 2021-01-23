use std::convert::TryFrom;

use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::Error;
use sqlx_core::Result;

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

impl TryFrom<u8> for TransactionStatus {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            b'I' => Ok(TransactionStatus::Idle),
            b'T' => Ok(TransactionStatus::Transaction),
            b'E' => Ok(TransactionStatus::Error),

            status => {
                return Err(Error::configuration_msg(format!(
                    "unknown transaction status: {:?}",
                    status as char,
                )));
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct ReadyForQuery {
    pub transaction_status: TransactionStatus,
}

impl Deserialize<'_, ()> for ReadyForQuery {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let transaction_status = TransactionStatus::try_from(buf[0])?;

        Ok(Self { transaction_status })
    }
}
