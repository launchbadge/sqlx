use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, Offset, Timelike};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
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
        matches!(ty.0.ty, DataType::DateTimeOffsetN)
    }
}

/// Split the time into days from Gregorian calendar, seconds and nanoseconds
/// as required for DateTime2
fn split_time(date_time: &NaiveDateTime) -> (i32, u32, u32) {
    let mut days = date_time.num_days_from_ce() - 1;
    let mut seconds = date_time.num_seconds_from_midnight();
    let mut ns = date_time.nanosecond();

    // this date format cannot encode anything outside of 0000-01-01 to 9999-12-31
    // so it's best to do some bounds-checking
    if days < 0 {
        days = 0;
        seconds = 0;
        ns = 0;
    } else if days > 3652058 {
        // corresponds to 9999-12-31, the highest plausible value for YYYY-MM-DD
        days = 3652058;
        seconds = 59 + 59 * 60 + 23 * 3600;
        ns = 999999900
    }
    (days, seconds, ns)
}

fn encode_date_time2(datetime: &NaiveDateTime) -> [u8; 8] {
    let (days, seconds, ns) = split_time(datetime);

    // always use full scale, 7 digits for nanoseconds,
    // requiring 5 bytes for seconds + nanoseconds combined
    let mut date = [0u8; 8];
    let ns_total = (seconds as i64) * 1_000_000_000 + ns as i64;
    let t = ns_total / 100;
    for i in 0..5 {
        date[i] = (t >> i * 8) as u8;
    }
    LittleEndian::write_i24(&mut date[5..8], days);
    date
}

/// Encodes DateTime objects for transfer over the wire
impl Encode<'_, Mssql> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let encoded = encode_date_time2(self);
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
fn decode_time(scale: u8, data: &[u8]) -> (u32, u32) {
    let mut acc = 0u64;
    for i in (0..data.len()).rev() {
        acc <<= 8;
        acc |= data[i] as u64;
    }
    acc *= 10u64.pow(9u32 - scale as u32);
    let seconds = acc / 1_000_000_000;
    let ns = acc % 1_000_000_000;
    (seconds as u32, ns as u32)
}

fn decode_datetime2(scale: u8, bytes: &[u8]) -> NaiveDateTime {
    let timesize = bytes.len() - 3;

    let days_from_ce = LittleEndian::read_i24(&bytes[timesize..]);
    let day = chrono::NaiveDate::from_num_days_from_ce(days_from_ce + 1);

    let (seconds, nanoseconds) = decode_time(scale, &bytes[0..timesize]);
    let time = chrono::NaiveTime::from_num_seconds_from_midnight(seconds, nanoseconds);

    day.and_time(time)
}

/// Decodes DateTime2N values received from the server
impl Decode<'_, Mssql> for NaiveDateTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        Ok(decode_datetime2(value.type_info.0.scale, bytes))
    }
}

impl Decode<'_, Mssql> for DateTime<FixedOffset> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let bytes = value.as_bytes()?;
        let naive = decode_datetime2(value.type_info.0.scale, &bytes[..bytes.len() - 2]);
        let offset = LittleEndian::read_i16(&bytes[bytes.len() - 2..]);
        Ok(DateTime::from_utc(naive, FixedOffset::east(offset as i32)))
    }
}
