use std::borrow::Cow;

pub(crate) use sqlx_core::value::*;

use crate::error::{BoxDynError, Error};
use crate::{Mssql, MssqlTypeInfo};

/// Internal storage for an MSSQL value, decoupled from tiberius lifetimes.
#[derive(Debug, Clone)]
pub(crate) enum MssqlData {
    Null,
    Bool(bool),
    U8(u8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Binary(Vec<u8>),
    #[cfg(feature = "chrono")]
    NaiveDateTime(chrono::NaiveDateTime),
    #[cfg(feature = "chrono")]
    NaiveDate(chrono::NaiveDate),
    #[cfg(feature = "chrono")]
    NaiveTime(chrono::NaiveTime),
    #[cfg(feature = "chrono")]
    DateTimeFixedOffset(chrono::DateTime<chrono::FixedOffset>),
    #[cfg(feature = "uuid")]
    Uuid(uuid::Uuid),
    #[cfg(feature = "rust_decimal")]
    Decimal(rust_decimal::Decimal),
    #[cfg(feature = "time")]
    TimeDate(time::Date),
    #[cfg(feature = "time")]
    TimeTime(time::Time),
    #[cfg(feature = "time")]
    TimePrimitiveDateTime(time::PrimitiveDateTime),
    #[cfg(feature = "time")]
    TimeOffsetDateTime(time::OffsetDateTime),
    #[cfg(feature = "bigdecimal")]
    BigDecimal(bigdecimal::BigDecimal),
}

/// Implementation of [`Value`] for MSSQL.
#[derive(Debug, Clone)]
pub struct MssqlValue {
    pub(crate) data: MssqlData,
    pub(crate) type_info: MssqlTypeInfo,
}

/// Implementation of [`ValueRef`] for MSSQL.
#[derive(Debug, Clone)]
pub struct MssqlValueRef<'r> {
    pub(crate) data: &'r MssqlData,
    pub(crate) type_info: MssqlTypeInfo,
}

impl<'r> MssqlValueRef<'r> {
    pub(crate) fn as_str(&self) -> Result<&'r str, BoxDynError> {
        match self.data {
            MssqlData::String(ref s) => Ok(s.as_str()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected string, got {:?}", self.data).into()),
        }
    }

    pub(crate) fn as_bytes(&self) -> Result<&'r [u8], BoxDynError> {
        match self.data {
            MssqlData::Binary(ref b) => Ok(b.as_slice()),
            MssqlData::String(ref s) => Ok(s.as_bytes()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected binary, got {:?}", self.data).into()),
        }
    }
}

impl Value for MssqlValue {
    type Database = Mssql;

    fn as_ref(&self) -> MssqlValueRef<'_> {
        MssqlValueRef {
            data: &self.data,
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, MssqlTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        matches!(self.data, MssqlData::Null)
    }
}

impl<'r> ValueRef<'r> for MssqlValueRef<'r> {
    type Database = Mssql;

    fn to_owned(&self) -> MssqlValue {
        MssqlValue {
            data: self.data.clone(),
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, MssqlTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        matches!(self.data, MssqlData::Null)
    }
}

/// Convert a `tiberius::ColumnData` into our owned `MssqlData`.
pub(crate) fn column_data_to_mssql_data(
    data: &tiberius::ColumnData<'_>,
) -> Result<MssqlData, Error> {
    match data {
        tiberius::ColumnData::U8(Some(v)) => Ok(MssqlData::U8(*v)),
        tiberius::ColumnData::I16(Some(v)) => Ok(MssqlData::I16(*v)),
        tiberius::ColumnData::I32(Some(v)) => Ok(MssqlData::I32(*v)),
        tiberius::ColumnData::I64(Some(v)) => Ok(MssqlData::I64(*v)),
        tiberius::ColumnData::F32(Some(v)) => Ok(MssqlData::F32(*v)),
        tiberius::ColumnData::F64(Some(v)) => Ok(MssqlData::F64(*v)),
        tiberius::ColumnData::Bit(Some(v)) => Ok(MssqlData::Bool(*v)),
        tiberius::ColumnData::String(Some(v)) => Ok(MssqlData::String(v.to_string())),
        tiberius::ColumnData::Binary(Some(v)) => Ok(MssqlData::Binary(v.to_vec())),

        #[cfg(feature = "chrono")]
        tiberius::ColumnData::DateTime2(Some(dt2)) => {
            let date = chrono_date_from_days(dt2.date().days() as i64, 1)?;
            let ns = dt2.time().increments() as i64
                * 10i64.pow(9u32.saturating_sub(dt2.time().scale() as u32));
            // infallible: (0,0,0) is always valid
            let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                + chrono::Duration::nanoseconds(ns);
            Ok(MssqlData::NaiveDateTime(chrono::NaiveDateTime::new(date, time)))
        }
        #[cfg(feature = "chrono")]
        tiberius::ColumnData::DateTime(Some(dt)) => {
            let date = chrono_date_from_days(dt.days() as i64, 1900)?;
            let ns = dt.seconds_fragments() as i64 * 1_000_000_000i64 / 300;
            // infallible: (0,0,0) is always valid
            let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                + chrono::Duration::nanoseconds(ns);
            Ok(MssqlData::NaiveDateTime(chrono::NaiveDateTime::new(date, time)))
        }
        #[cfg(feature = "chrono")]
        tiberius::ColumnData::SmallDateTime(Some(dt)) => {
            let date = chrono_date_from_days(dt.days() as i64, 1900)?;
            let seconds = dt.seconds_fragments() as u32 * 60;
            let time = chrono::NaiveTime::from_num_seconds_from_midnight_opt(seconds, 0)
                .ok_or_else(|| {
                    Error::Protocol(
                        format!(
                            "invalid SmallDateTime seconds: {seconds} exceeds seconds-in-a-day"
                        )
                        .into(),
                    )
                })?;
            Ok(MssqlData::NaiveDateTime(chrono::NaiveDateTime::new(date, time)))
        }
        #[cfg(feature = "chrono")]
        tiberius::ColumnData::Date(Some(d)) => {
            Ok(MssqlData::NaiveDate(chrono_date_from_days(d.days() as i64, 1)?))
        }
        #[cfg(feature = "chrono")]
        tiberius::ColumnData::Time(Some(t)) => {
            let ns =
                t.increments() as i64 * 10i64.pow(9u32.saturating_sub(t.scale() as u32));
            // infallible: (0,0,0) is always valid
            let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                + chrono::Duration::nanoseconds(ns);
            Ok(MssqlData::NaiveTime(time))
        }
        #[cfg(feature = "chrono")]
        tiberius::ColumnData::DateTimeOffset(Some(dto)) => {
            let date = chrono_date_from_days(dto.datetime2().date().days() as i64, 1)?;
            let ns = dto.datetime2().time().increments() as i64
                * 10i64.pow(9u32.saturating_sub(dto.datetime2().time().scale() as u32));
            // infallible: (0,0,0) is always valid
            let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                + chrono::Duration::nanoseconds(ns);
            let naive = chrono::NaiveDateTime::new(date, time);
            let offset_secs = dto.offset() as i32 * 60;
            let fixed_offset = chrono::FixedOffset::east_opt(offset_secs).ok_or_else(|| {
                Error::Protocol(
                    format!("invalid timezone offset: {offset_secs} seconds").into(),
                )
            })?;
            let dt = naive.and_local_timezone(fixed_offset).single().ok_or_else(|| {
                Error::Protocol(
                    format!(
                        "ambiguous or invalid local time for offset {offset_secs}s"
                    )
                    .into(),
                )
            })?;
            Ok(MssqlData::DateTimeFixedOffset(dt))
        }

        #[cfg(feature = "uuid")]
        tiberius::ColumnData::Guid(Some(v)) => Ok(MssqlData::Uuid(*v)),

        #[cfg(feature = "rust_decimal")]
        tiberius::ColumnData::Numeric(Some(n)) => {
            Ok(MssqlData::Decimal(rust_decimal::Decimal::from_i128_with_scale(
                n.value(),
                n.scale() as u32,
            )))
        }

        #[cfg(all(feature = "time", not(feature = "chrono")))]
        tiberius::ColumnData::Date(Some(d)) => {
            Ok(MssqlData::TimeDate(time_date_from_days(d.days() as u64, 1)?))
        }
        #[cfg(all(feature = "time", not(feature = "chrono")))]
        tiberius::ColumnData::Time(Some(t)) => {
            let ns = t.increments() as u64
                * 10u64.pow(9u32.saturating_sub(t.scale() as u32));
            Ok(MssqlData::TimeTime(time_from_sec_fragments(ns)?))
        }
        #[cfg(all(feature = "time", not(feature = "chrono")))]
        tiberius::ColumnData::DateTime2(Some(dt2)) => {
            let date = time_date_from_days(dt2.date().days() as u64, 1)?;
            let ns = dt2.time().increments() as u64
                * 10u64.pow(9u32.saturating_sub(dt2.time().scale() as u32));
            let time = time_from_sec_fragments(ns)?;
            Ok(MssqlData::TimePrimitiveDateTime(time::PrimitiveDateTime::new(date, time)))
        }
        #[cfg(all(feature = "time", not(feature = "chrono")))]
        tiberius::ColumnData::DateTime(Some(dt)) => {
            let date = time_date_from_days(dt.days() as u64, 1900)?;
            let ns = dt.seconds_fragments() as u64 * 1_000_000_000u64 / 300;
            let time = time_from_sec_fragments(ns)?;
            Ok(MssqlData::TimePrimitiveDateTime(time::PrimitiveDateTime::new(date, time)))
        }
        #[cfg(all(feature = "time", not(feature = "chrono")))]
        tiberius::ColumnData::SmallDateTime(Some(dt)) => {
            let date = time_date_from_days(dt.days() as u64, 1900)?;
            let seconds = dt.seconds_fragments() as u64 * 60;
            let time = time_from_sec_fragments(seconds * 1_000_000_000)?;
            Ok(MssqlData::TimePrimitiveDateTime(time::PrimitiveDateTime::new(date, time)))
        }
        #[cfg(all(feature = "time", not(feature = "chrono")))]
        tiberius::ColumnData::DateTimeOffset(Some(dto)) => {
            let date = time_date_from_days(dto.datetime2().date().days() as u64, 1)?;
            let ns = dto.datetime2().time().increments() as u64
                * 10u64.pow(9u32.saturating_sub(dto.datetime2().time().scale() as u32));
            let time = time_from_sec_fragments(ns)?;
            let naive = time::PrimitiveDateTime::new(date, time);
            let offset_secs = dto.offset() as i32 * 60;
            let offset = time::UtcOffset::from_whole_seconds(offset_secs).map_err(|_| {
                Error::Protocol(
                    format!("invalid UTC offset: {offset_secs} seconds").into(),
                )
            })?;
            Ok(MssqlData::TimeOffsetDateTime(naive.assume_offset(offset)))
        }

        #[cfg(all(feature = "bigdecimal", not(feature = "rust_decimal")))]
        tiberius::ColumnData::Numeric(Some(n)) => {
            use bigdecimal::num_bigint::BigInt;
            Ok(MssqlData::BigDecimal(bigdecimal::BigDecimal::new(
                BigInt::from(n.value()),
                n.scale() as i64,
            )))
        }

        // All None variants represent SQL NULL
        tiberius::ColumnData::U8(None)
        | tiberius::ColumnData::I16(None)
        | tiberius::ColumnData::I32(None)
        | tiberius::ColumnData::I64(None)
        | tiberius::ColumnData::F32(None)
        | tiberius::ColumnData::F64(None)
        | tiberius::ColumnData::Bit(None)
        | tiberius::ColumnData::String(None)
        | tiberius::ColumnData::Guid(None)
        | tiberius::ColumnData::Binary(None)
        | tiberius::ColumnData::Numeric(None)
        | tiberius::ColumnData::Xml(None)
        | tiberius::ColumnData::DateTime(None)
        | tiberius::ColumnData::SmallDateTime(None)
        | tiberius::ColumnData::DateTime2(None)
        | tiberius::ColumnData::DateTimeOffset(None)
        | tiberius::ColumnData::Date(None)
        | tiberius::ColumnData::Time(None) => Ok(MssqlData::Null),

        // Unhandled Some(...) variant — real data the driver can't convert
        other => {
            let debug = format!("{other:?}");
            let truncated = if debug.len() > 200 { &debug[..200] } else { &debug };
            Err(Error::Protocol(
                format!("unsupported tiberius ColumnData variant: {truncated}").into(),
            ))
        }
    }
}

/// Convert days since `start_year`-01-01 to a `time::Date`.
#[cfg(feature = "time")]
fn time_date_from_days(days: u64, start_year: i32) -> Result<time::Date, Error> {
    let start = time::Date::from_ordinal_date(start_year, 1).map_err(|_| {
        Error::Protocol(format!("invalid start year for date: {start_year}").into())
    })?;
    start
        .checked_add(time::Duration::days(days as i64))
        .ok_or_else(|| {
            Error::Protocol(
                format!("date overflow: {days} days from {start_year}-01-01").into(),
            )
        })
}

/// Convert nanoseconds-since-midnight to a `time::Time`.
#[cfg(feature = "time")]
fn time_from_sec_fragments(nanoseconds: u64) -> Result<time::Time, Error> {
    const NANOS_PER_DAY: u64 = 86_400_000_000_000;
    if nanoseconds >= NANOS_PER_DAY {
        return Err(Error::Protocol(
            format!(
                "time nanoseconds out of range: {nanoseconds} (must be < {NANOS_PER_DAY})"
            )
            .into(),
        ));
    }
    // After the bounds check, hours is 0..=23, minutes 0..=59, seconds 0..=59,
    // so the `as u8` casts and `from_hms_nano` are all infallible.
    let hours = (nanoseconds / 3_600_000_000_000) as u8;
    let remaining = nanoseconds % 3_600_000_000_000;
    let minutes = (remaining / 60_000_000_000) as u8;
    let remaining = remaining % 60_000_000_000;
    let seconds = (remaining / 1_000_000_000) as u8;
    let nanos = (remaining % 1_000_000_000) as u32;
    time::Time::from_hms_nano(hours, minutes, seconds, nanos).map_err(|_| {
        Error::Protocol(
            format!("invalid time: {hours:02}:{minutes:02}:{seconds:02}.{nanos:09}")
                .into(),
        )
    })
}

/// Convert days since `start_year`-01-01 to a `chrono::NaiveDate`.
#[cfg(feature = "chrono")]
fn chrono_date_from_days(days: i64, start_year: i32) -> Result<chrono::NaiveDate, Error> {
    let start = chrono::NaiveDate::from_ymd_opt(start_year, 1, 1).ok_or_else(|| {
        Error::Protocol(format!("invalid start year for date: {start_year}").into())
    })?;
    start
        .checked_add_signed(chrono::Duration::days(days))
        .ok_or_else(|| {
            Error::Protocol(
                format!("date overflow: {days} days from {start_year}-01-01").into(),
            )
        })
}
