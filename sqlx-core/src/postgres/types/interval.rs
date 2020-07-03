use std::mem;

use byteorder::{NetworkEndian, ReadBytesExt};

#[cfg(any(feature = "chrono", feature = "time"))]
use std::convert::TryFrom;

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

impl PgInterval {
    /// Convert a `std::time::Duration` object to a `PgInterval` object but truncate the remaining nanoseconds.
    ///
    /// Returns an error if there is a microseconds overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use sqlx_core::postgres::types::PgInterval;
    /// let interval = PgInterval::truncate_nanos_std(std::time::Duration::from_secs(3_600)).unwrap();
    /// assert_eq!(interval, PgInterval { months: 0, days: 0, microseconds: 3_600_000_000 });
    /// ```
    pub fn truncate_nanos_std(value: std::time::Duration) -> Result<Self, BoxDynError> {
        let microseconds = i64::try_from(value.as_micros())?;
        Ok(Self {
            months: 0,
            days: 0,
            microseconds,
        })
    }
    /// Convert a `time::Duration` object to a `PgInterval` object but truncate the remaining nanoseconds.
    ///
    /// Returns an error if there is a microseconds overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use sqlx_core::postgres::types::PgInterval;
    /// let interval = PgInterval::truncate_nanos_time(time::Duration::seconds(3_600)).unwrap();
    /// assert_eq!(interval, PgInterval { months: 0, days: 0, microseconds: 3_600_000_000 });
    /// ```
    #[cfg(feature = "time")]
    pub fn truncate_nanos_time(value: time::Duration) -> Result<Self, BoxDynError> {
        let microseconds = i64::try_from(value.whole_microseconds())?;
        Ok(Self {
            months: 0,
            days: 0,
            microseconds,
        })
    }

    /// Convert a `chrono::Duration` object to a `PgInterval` object but truncates the remaining nanoseconds.
    /// Returns an error if there is a microseconds overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use sqlx_core::postgres::types::PgInterval;
    /// let interval = PgInterval::truncate_nanos_chrono(chrono::Duration::seconds(3_600)).unwrap();
    /// assert_eq!(interval, PgInterval { months: 0, days: 0, microseconds: 3_600_000_000 });
    /// ```
    #[cfg(feature = "chrono")]
    pub fn truncate_nanos_chrono(value: chrono::Duration) -> Result<Self, BoxDynError> {
        let microseconds = value.num_microseconds().ok_or("Microseconds overflow")?;
        Ok(Self {
            months: 0,
            days: 0,
            microseconds,
        })
    }
}

impl Type<Postgres> for std::time::Duration {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL
    }
}

impl Type<Postgres> for [std::time::Duration] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL_ARRAY
    }
}

impl Encode<'_, Postgres> for std::time::Duration {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let pg_interval =
            PgInterval::try_from(*self).expect("Failed to encode std::time::Duration");
        pg_interval.encode_by_ref(buf)
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

impl TryFrom<std::time::Duration> for PgInterval {
    type Error = BoxDynError;

    /// Convert a `std::time::Duration` to a `PgInterval`
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// microsecond overflow
    ///
    /// To do lossy conversion use `PgInterval::truncate_nanos_std()`.
    fn try_from(value: std::time::Duration) -> Result<Self, BoxDynError> {
        match value.as_nanos() {
            n if n % 1000 != 0 => {
                Err("PostgreSQL INTERVAL does not support nanoseconds precision".into())
            }
            _ => Ok(Self {
                months: 0,
                days: 0,
                microseconds: i64::try_from(value.as_micros())?,
            }),
        }
    }
}

#[cfg(feature = "chrono")]
impl Type<Postgres> for chrono::Duration {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL
    }
}

#[cfg(feature = "chrono")]
impl Type<Postgres> for [chrono::Duration] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL_ARRAY
    }
}

#[cfg(feature = "chrono")]
impl Encode<'_, Postgres> for chrono::Duration {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let pg_interval = PgInterval::try_from(*self).expect("Failed to encode chrono::Duration");
        pg_interval.encode_by_ref(buf)
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<chrono::Duration> for PgInterval {
    type Error = BoxDynError;

    /// Convert a `chrono::Duration` to a `PgInterval`
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// microsecond or nanosecond overflow
    ///
    /// To do a lossy conversion use `PgInterval::truncate_nanos_chrono()`.
    fn try_from(value: chrono::Duration) -> Result<Self, BoxDynError> {
        let microseconds = value.num_microseconds().ok_or("Microseconds overflow")?;
        match value
            .checked_sub(&chrono::Duration::microseconds(microseconds))
            .ok_or("Microseconds overflow")?
            .num_nanoseconds()
            .ok_or("Nanoseconds overflow")?
        {
            0 => Ok(Self {
                months: 0,
                days: 0,
                microseconds,
            }),
            _ => Err("PostgreSQL INTERVAL does not support nanoseconds precision".into()),
        }
    }
}

#[cfg(feature = "time")]
impl Type<Postgres> for time::Duration {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL
    }
}

#[cfg(feature = "time")]
impl Type<Postgres> for [time::Duration] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::INTERVAL_ARRAY
    }
}

#[cfg(feature = "time")]
impl Encode<'_, Postgres> for time::Duration {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let pg_interval = PgInterval::try_from(*self).expect("Failed to encode time::Duration");
        pg_interval.encode_by_ref(buf)
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

#[cfg(feature = "time")]
impl TryFrom<time::Duration> for PgInterval {
    type Error = BoxDynError;

    /// Convert a `time::Duration` to a `PgInterval`
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// microsecond overflow
    ///
    /// To do a lossy conversion use `PgInterval::time_truncate_nanos()`.
    fn try_from(value: time::Duration) -> Result<Self, BoxDynError> {
        let microseconds = i64::try_from(value.whole_microseconds())?;
        match value
            .checked_sub(time::Duration::microseconds(microseconds))
            .ok_or("Microseconds overflow")?
            .subsec_nanoseconds()
        {
            0 => Ok(Self {
                months: 0,
                days: 0,
                microseconds,
            }),
            _ => Err("PostgreSQL INTERVAL does not support nanoseconds precision".into()),
        }
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

#[test]
fn test_pginterval_std() {
    let interval = PgInterval {
        days: 0,
        months: 0,
        microseconds: 27_000,
    };
    assert_eq!(
        &PgInterval::try_from(std::time::Duration::from_micros(27_000)).unwrap(),
        &interval
    );
}

#[test]
#[cfg(feature = "chrono")]
fn test_pginterval_chrono() {
    let interval = PgInterval {
        days: 0,
        months: 0,
        microseconds: 27_000,
    };
    assert_eq!(
        &PgInterval::try_from(chrono::Duration::microseconds(27_000)).unwrap(),
        &interval
    );
}

#[test]
#[cfg(feature = "time")]
fn test_pginterval_time() {
    let interval = PgInterval {
        days: 0,
        months: 0,
        microseconds: 27_000,
    };
    assert_eq!(
        &PgInterval::try_from(time::Duration::microseconds(27_000)).unwrap(),
        &interval
    );
}
