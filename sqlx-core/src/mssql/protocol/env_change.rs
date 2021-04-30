use bytes::{Buf, Bytes};

use crate::error::Error;
use crate::mssql::io::MssqlBufExt;

#[derive(Debug, PartialEq)]
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
    BeginTransaction(u64),
    CommitTransaction(u64),
    RollbackTransaction(u64),
    EnlistDtcTransaction,
    DefectTransaction,
    RealTimeLogShipping,
    PromoteTransaction,
    TransactionManagerAddress,
    TransactionEnded,
    ResetConnectionCompletionAck,
    LoginRequestUserNameAck,

    // TDS 7.4+
    RoutingInformation(String, u16),
}

impl EnvChange {
    pub(crate) fn get(buf: &mut Bytes) -> Result<Self, Error> {
        // FIXME: All of the get_* calls below can panic if we didn't get enough data from the
        // server. Should add error handling to fail gracefully.
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
            8 => EnvChange::BeginTransaction(data.get_b_varbyte().get_u64_le()),

            9 => {
                let _ = data.get_u8();
                EnvChange::CommitTransaction(data.get_u64_le())
            }

            10 => {
                let _ = data.get_u8();
                EnvChange::RollbackTransaction(data.get_u64_le())
            }

            20 => {
                let _value_len = data.get_u16_le();
                let protocol = data.get_u8();
                if protocol != 0 /* TCP */ {
                    return Err(
                        err_protocol!("unexpected protocol {} in Routing ENVCHANGE", protocol));
                }
                let new_port = data.get_u16_le();
                let new_host = data.get_us_varchar()?;
                let old_value = data.get_u16_le();
                if old_value != 0 {
                    return Err(
                        err_protocol!("unexpected old value {} in Routing ENVCHANGE", old_value));
                }
                EnvChange::RoutingInformation(new_host, new_port)
            }

            _ => {
                return Err(err_protocol!("unexpected value {} for ENVCHANGE Type", ty));
            }
        })
    }
}

#[test]
fn test_envchange_routing() {
    let buf = vec![
        0x14, 0x00, // Data size
        0x14, // EnvChange type.
        0x00, 0x00, // Value len (ignored).
        0x00, // Protocol (TCP).
        0x34, 0x12, // New port.
        0x05, 0x00, b'h', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0, // New host.
        0x00, 0x00, // Old value
    ];
    let mut bytes = Bytes::from(buf);
    let ec = EnvChange::get(&mut bytes).unwrap();
    assert_eq!(EnvChange::RoutingInformation("hello".to_owned(), 4660), ec);
}
