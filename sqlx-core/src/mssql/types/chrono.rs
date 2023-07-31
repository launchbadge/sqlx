use byteorder::{ByteOrder, LittleEndian};
use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Offset, Timelike,
};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{self, BoxDynError};
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

/// Provides conversion of chrono::DateTime (UTC) to MS SQL DateTime2N
///
/// Note that MS SQL has a number of DateTime-related types and conversion
/// might not work.
/// During encoding, values are always encoded with the best possible
/// precision, which uses 7 digits for nanoseconds.
impl Type<Mssql> for NaiveDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo {
            scale: 7,
            ty: DataType::DateTime2N,
            size: 8,
            collation: None,
            precision: 0,
        })
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::DateTime2N)
    }
}

/// Provides conversion of chrono::NaiveDate to MS SQL Date
impl Type<Mssql> for NaiveDate {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo {
            scale: 0,
            ty: DataType::DateN,
            size: 3,
            collation: None,
            precision: 10,
        })
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::DateN)
    }
}

/// Provides conversion of chrono::NaiveTime to MS SQL Time
impl Type<Mssql> for NaiveTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo {
            scale: 7,
            ty: DataType::TimeN,
            size: 5,
            collation: None,
            precision: 0,
        })
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.0.ty, DataType::TimeN)
    }
}

impl<T> Type<Mssql> for DateTime<T>
where
    T: chrono::TimeZone,
{
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo {
            scale: 7,
            ty: DataType::DateTimeOffsetN,
            size: 8,
            collation: None,
            precision: 34,
        })
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.0.ty,
            DataType::DateTimeOffsetN
                | DataType::DateTime
                | DataType::DateTimeN
                | DataType::SmallDateTime
        )
    }
}

fn encode_date(date: &NaiveDate) -> [u8; 3] {
    let days = date.num_days_from_ce() - 1;
    let mut date = [0u8; 3];
    LittleEndian::write_i24(&mut date, days);
    date
}

fn encode_date_time2(datetime: &NaiveDateTime) -> [u8; 8] {
    let date = datetime.date();
    let time = datetime.time();
    let mut buf = [0u8; 8];
    buf[0..5].copy_from_slice(&encode_time(&time));
    buf[5..8].copy_from_slice(&encode_date(&date));
    buf
}

fn encode_time(time: &NaiveTime) -> [u8; 5] {
    // always use full scale, 7 digits for nanoseconds,
    // requiring 5 bytes for seconds + nanoseconds combined
    let seconds = time.num_seconds_from_midnight();
    let ns = time.nanosecond();
    let mut buf = [0u8; 5];
    let total = i64::from(seconds) * 10_000_000 + i64::from(ns / 100);
    buf.copy_from_slice(&total.to_le_bytes()[0..5]);
    buf
}

/// Encodes DateTime objects for transfer over the wire
impl Encode<'_, Mssql> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let encoded = encode_date_time2(self);
        buf.extend_from_slice(&encoded);
        IsNull::No
    }
}

/// Encodes Date objects for transfer over the wire
impl Encode<'_, Mssql> for NaiveDate {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let encoded = encode_date(self);
        buf.extend_from_slice(&encoded);
        IsNull::No
    }
}

/// Encodes Time objects for transfer over the wire
impl Encode<'_, Mssql> for NaiveTime {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let encoded = encode_time(self);
        buf.extend_from_slice(&encoded);
        IsNull::No
    }
}

impl<T> Encode<'_, Mssql> for DateTime<T>
where
    T: chrono::TimeZone,
{
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(&encode_date_time2(&self.naive_utc()));
        let from_utc = self.offset().fix().local_minus_utc();
        let mut encoded_offset: [u8; 2] = [0, 0];
        LittleEndian::write_i16(&mut encoded_offset, (from_utc / 60) as i16);
        buf.extend_from_slice(&encoded_offset);
        IsNull::No
    }
}

/// Determines seconds since midnight and nanoseconds since the last second
fn decode_time(scale: u8, data: &[u8]) -> error::Result<NaiveTime> {
    let mut acc = 0u64;
    for i in (0..data.len()).rev() {
        acc <<= 8;
        acc |= data[i] as u64;
    }
    acc *= 10u64.pow(9u32 - scale as u32);
    let seconds = u32::try_from(acc / 1_000_000_000).unwrap();
    let ns = u32::try_from(acc % 1_000_000_000).unwrap();

    chrono::NaiveTime::from_num_seconds_from_midnight_opt(seconds, ns)
        .ok_or_else(|| err_protocol!("invalid time: seconds={} nanoseconds={}", seconds, ns))
}

fn decode_date(bytes: &[u8]) -> error::Result<NaiveDate> {
    let days_from_ce = LittleEndian::read_i24(&bytes);
    chrono::NaiveDate::from_num_days_from_ce_opt(days_from_ce + 1)
        .ok_or_else(|| err_protocol!("invalid days offset in date: {}", days_from_ce))
}

fn decode_datetime2(scale: u8, bytes: &[u8]) -> Result<NaiveDateTime, BoxDynError> {
    let timesize = bytes.len() - 3;
    let time = decode_time(scale, &bytes[0..timesize])?;
    let day = decode_date(&bytes[timesize..])?;
    Ok(day.and_time(time))
}

// Decodes a DATETIME (the old TSQL date time type)
fn decode_datetime(bytes: &[u8]) -> Result<DateTime<FixedOffset>, BoxDynError> {
    let (date_bytes, time_bytes) = bytes.split_at(4);
    let date_bytes = <[u8; 4]>::try_from(date_bytes)?;
    let time_bytes = <[u8; 4]>::try_from(time_bytes)?;
    let days = i32::from_le_bytes(date_bytes); // days since January 1, 1900
    let t = u32::from_le_bytes(time_bytes); // three-hundredths of a second since midnight
    let naive_date =
        NaiveDate::from_ymd_opt(1900, 1, 1).unwrap() + chrono::Duration::days(i64::from(days));
    let millis = i64::from(t) * 1000 / 300;
    let naive_datetime =
        naive_date.and_time(NaiveTime::default()) + chrono::Duration::milliseconds(millis);
    Ok(naive_datetime.and_utc().fixed_offset())
}

/// Decodes DateTime2N values received from the server
impl Decode<'_, Mssql> for NaiveDateTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        match value.type_info.0.ty {
            DataType::SmallDateTime => todo!(),
            DataType::DateTimeN => todo!(),
            DataType::DateTime2N => decode_datetime2(value.type_info.0.scale, bytes),
            DataType::DateTimeOffsetN => todo!(),
            _ => Err("unsupported datetime type".into()),
        }
    }
}

/// Decodes Date values received from the server
impl Decode<'_, Mssql> for NaiveDate {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        let date = decode_date(bytes)?;
        Ok(date)
    }
}

/// Decodes Time values received from the server
impl Decode<'_, Mssql> for NaiveTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        let time = decode_time(value.type_info.0.scale, bytes)?;
        Ok(time)
    }
}

impl Decode<'_, Mssql> for DateTime<FixedOffset> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        let scale = value.type_info.0.scale;
        match value.type_info.0.ty {
            DataType::SmallDateTime => todo!(),
            DataType::DateTimeN => decode_datetime(bytes),
            DataType::DateTimeOffsetN => decode_datetimeoffset(scale, bytes),
            _ => Err("unsupported datetime+offset type".into()),
        }
    }
}

fn decode_datetimeoffset(scale: u8, bytes: &[u8]) -> Result<DateTime<FixedOffset>, BoxDynError> {
    let naive = decode_datetime2(scale, &bytes[..bytes.len() - 2])?;
    let offset = LittleEndian::read_i16(&bytes[bytes.len() - 2..]);
    let offset_parsed = FixedOffset::east_opt(i32::from(offset)).ok_or_else(|| {
        Box::new(err_protocol!("invalid offset {} in DateTime2N", offset)) as BoxDynError
    })?;
    Ok(DateTime::from_utc(naive, offset_parsed))
}
