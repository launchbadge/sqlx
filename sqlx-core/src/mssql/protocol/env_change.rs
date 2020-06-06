use bytes::{Buf, Bytes};

use crate::error::Error;
use crate::io::Decode;
use crate::mssql::io::MsSqlBufExt;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum EnvChange {
    Database(String),
    Language(String),
    CharacterSet(String),
    PacketSize(String),
    UnicodeDataSortingLocalId(String),
    UnicodeDataSortingComparisonFlags(String),
    SqlCollation(Bytes),

    // TDS 7.2+
    BeginTransaction,
    CommitTransaction,
    RollbackTransaction,
    EnlistDtcTransaction,
    DefectTransaction,
    RealTimeLogShipping,
    PromoteTransaction,
    TransactionManagerAddress,
    TransactionEnded,
    ResetConnectionCompletionAck,
    LoginRequestUserNameAck,

    // TDS 7.4+
    RoutingInformation,
}

impl EnvChange {
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, Error> {
        let len = buf.get_u16_le();
        let ty = buf.get_u8();
        let mut data = buf.split_to((len - 1) as usize);

        Ok(match ty {
            1 => EnvChange::Database(data.get_b_varchar()?),
            2 => EnvChange::Language(data.get_b_varchar()?),
            3 => EnvChange::CharacterSet(data.get_b_varchar()?),
            4 => EnvChange::PacketSize(data.get_b_varchar()?),
            5 => EnvChange::UnicodeDataSortingLocalId(data.get_b_varchar()?),
            6 => EnvChange::UnicodeDataSortingComparisonFlags(data.get_b_varchar()?),
            7 => EnvChange::SqlCollation(data.get_b_varbyte()),

            _ => {
                return Err(err_protocol!("unexpected value {} for ENVCHANGE Type", ty));
            }
        })
    }
}
