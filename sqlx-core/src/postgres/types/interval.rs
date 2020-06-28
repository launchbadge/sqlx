use std::mem;

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use crate::types::Type;

/// PostgreSQL INTERVAL type binding
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PgInterval {
    pub months: i32,
    pub days: i32,
    pub microseconds: i64,
}

impl Type<Postgres> for PgInterval {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL
    }
}

impl Type<Postgres> for [PgInterval] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL_ARRAY
    }
}

impl<'de> Decode<'de, Postgres> for PgInterval {
    fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let mut buf = value.as_bytes()?;
                let microseconds = buf.read_i64::<NetworkEndian>()?;
                let days = buf.read_i32::<NetworkEndian>()?;
                let months = buf.read_i32::<NetworkEndian>()?;
                Ok(PgInterval {
                    months,
                    days,
                    microseconds,
                })
            }
            PgValueFormat::Text => Err("INTERVAL Text format unsuported".into()),
        }
    }
}

impl Encode<'_, Postgres> for PgInterval {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        if let IsNull::Yes = Encode::<Postgres>::encode(&self.microseconds, buf) {
            return IsNull::Yes;
        }
        if let IsNull::Yes = Encode::<Postgres>::encode(&self.days, buf) {
            return IsNull::Yes;
        }
        if let IsNull::Yes = Encode::<Postgres>::encode(&self.months, buf) {
            return IsNull::Yes;
        }
        IsNull::No
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

#[test]
fn test_encode_interval() {
    let mut buf = PgArgumentBuffer::default();

    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 0,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    buf.clear();

    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 1_000,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 0, 3, 232, 0, 0, 0, 0, 0, 0, 0, 0]);
    buf.clear();

    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 1_000_000,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 15, 66, 64, 0, 0, 0, 0, 0, 0, 0, 0]);
    buf.clear();

    let interval = PgInterval {
        months: 0,
        days: 0,
        microseconds: 3_600_000_000,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(
        &**buf,
        [0, 0, 0, 0, 214, 147, 164, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    buf.clear();

    let interval = PgInterval {
        months: 0,
        days: 1,
        microseconds: 0,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
    buf.clear();

    let interval = PgInterval {
        months: 1,
        days: 0,
        microseconds: 0,
    };
    assert!(matches!(
        Encode::<Postgres>::encode(&interval, &mut buf),
        IsNull::No
    ));
    assert_eq!(&**buf, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    buf.clear();
}
