use std::borrow::Cow;
use std::convert::TryInto;
use std::mem;

use byteorder::BigEndian;
use time::{date, offset, Date, NumericalDuration, OffsetDateTime, PrimitiveDateTime, Time};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::io::Buf;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::{PgValue, Postgres};
use crate::types::Type;

const POSTGRES_EPOCH: PrimitiveDateTime = date!(2000 - 1 - 1).midnight();

impl Type<Postgres> for Time {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIME, "TIME")
    }
}

impl Type<Postgres> for Date {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::DATE, "DATE")
    }
}

impl Type<Postgres> for PrimitiveDateTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIMESTAMP, "TIMESTAMP")
    }
}

impl Type<Postgres> for OffsetDateTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIMESTAMPTZ, "TIMESTAMPTZ")
    }
}

impl Type<Postgres> for [Time] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIME, "TIME[]")
    }
}

impl Type<Postgres> for [Date] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_DATE, "DATE[]")
    }
}

impl Type<Postgres> for [PrimitiveDateTime] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIMESTAMP, "TIMESTAMP[]")
    }
}

impl Type<Postgres> for [OffsetDateTime] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIMESTAMPTZ, "TIMESTAMPTZ[]")
    }
}

impl Type<Postgres> for Vec<Time> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIME, "TIME[]")
    }
}

impl Type<Postgres> for Vec<Date> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_DATE, "DATE[]")
    }
}

impl Type<Postgres> for Vec<PrimitiveDateTime> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIMESTAMP, "TIMESTAMP[]")
    }
}

impl Type<Postgres> for Vec<OffsetDateTime> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIMESTAMPTZ, "TIMESTAMPTZ[]")
    }
}

fn microseconds_since_midnight(time: Time) -> i64 {
    time.hour() as i64 * 60 * 60 * 1_000_000
        + time.minute() as i64 * 60 * 1_000_000
        + time.second() as i64 * 1_000_000
        + time.microsecond() as i64
}

fn from_microseconds_since_midnight(mut microsecond: u64) -> crate::Result<Postgres, Time> {
    #![allow(clippy::cast_possible_truncation)]

    microsecond %= 86_400 * 1_000_000;

    Time::try_from_hms_micro(
        (microsecond / 1_000_000 / 60 / 60) as u8,
        (microsecond / 1_000_000 / 60 % 60) as u8,
        (microsecond / 1_000_000 % 60) as u8,
        (microsecond % 1_000_000) as u32,
    )
    .map_err(|e| decode_err!("Time out of range for Postgres: {}", e))
}

impl<'de> Decode<'de, Postgres> for Time {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => {
                let micros: i64 = buf.get_i64::<BigEndian>()?;

                from_microseconds_since_midnight(micros as u64)
            }

            PgValue::Text(s) => {
                // If there are less than 9 digits after the decimal point
                // We need to zero-pad
                // TODO: Ask [time] to add a parse % for less-than-fixed-9 nanos

                let s = if s.len() < 20 {
                    Cow::Owned(format!("{:0<19}", s))
                } else {
                    Cow::Borrowed(s)
                };

                Time::parse(&*s, "%H:%M:%S.%N").map_err(crate::Error::decode)
            }
        }
    }
}

impl Encode<Postgres> for Time {
    fn encode(&self, buf: &mut Vec<u8>) {
        let micros = microseconds_since_midnight(*self);

        Encode::<Postgres>::encode(&micros, buf);
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<u64>()
    }
}

impl<'de> Decode<'de, Postgres> for Date {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => {
                let n: i32 = buf.get_i32::<BigEndian>()?;

                Ok(date!(2000 - 1 - 1) + n.days())
            }

            PgValue::Text(s) => Date::parse(s, "%Y-%m-%d").map_err(crate::Error::decode),
        }
    }
}

impl Encode<Postgres> for Date {
    fn encode(&self, buf: &mut Vec<u8>) {
        let days: i32 = (*self - date!(2000 - 1 - 1))
            .whole_days()
            .try_into()
            // TODO: How does Diesel handle this?
            .unwrap_or_else(|_| panic!("Date out of range for Postgres: {:?}", self));

        Encode::<Postgres>::encode(&days, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i32>()
    }
}

impl<'de> Decode<'de, Postgres> for PrimitiveDateTime {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => {
                let n: i64 = buf.get_i64::<BigEndian>()?;

                Ok(POSTGRES_EPOCH + n.microseconds())
            }

            // TODO: Try and fix duplication between here and MySQL
            PgValue::Text(s) => {
                // If there are less than 9 digits after the decimal point
                // We need to zero-pad
                // TODO: Ask [time] to add a parse % for less-than-fixed-9 nanos

                let s = if let Some(plus) = s.rfind('+') {
                    let mut big = String::from(&s[..plus]);

                    while big.len() < 31 {
                        big.push('0');
                    }

                    big.push_str(&s[plus..]);

                    Cow::Owned(big)
                } else if s.len() < 31 {
                    if s.contains('.') {
                        Cow::Owned(format!("{:0<30}", s))
                    } else {
                        Cow::Owned(format!("{}.000000000", s))
                    }
                } else {
                    Cow::Borrowed(s)
                };

                PrimitiveDateTime::parse(&*s, "%Y-%m-%d %H:%M:%S.%N").map_err(crate::Error::decode)
            }
        }
    }
}

impl Encode<Postgres> for PrimitiveDateTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        let micros: i64 = (*self - POSTGRES_EPOCH)
            .whole_microseconds()
            .try_into()
            .unwrap_or_else(|_| panic!("PrimitiveDateTime out of range for Postgres: {:?}", self));

        Encode::<Postgres>::encode(&micros, buf);
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'de> Decode<'de, Postgres> for OffsetDateTime {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        let primitive: PrimitiveDateTime = Decode::<Postgres>::decode(value)?;

        Ok(primitive.assume_utc())
    }
}

impl Encode<Postgres> for OffsetDateTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        let utc_dt = self.to_offset(offset!(UTC));
        let primitive_dt = PrimitiveDateTime::new(utc_dt.date(), utc_dt.time());

        Encode::<Postgres>::encode(&primitive_dt, buf);
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

#[cfg(test)]
use time::time;

#[test]
fn test_encode_time() {
    let mut buf = Vec::new();

    Encode::<Postgres>::encode(&time!(0:00), &mut buf);
    assert_eq!(buf, [0; 8]);
    buf.clear();

    // one second
    Encode::<Postgres>::encode(&time!(0:00:01), &mut buf);
    assert_eq!(buf, 1_000_000i64.to_be_bytes());
    buf.clear();

    // two hours
    Encode::<Postgres>::encode(&time!(2:00), &mut buf);
    let expected = 1_000_000i64 * 60 * 60 * 2;
    assert_eq!(buf, expected.to_be_bytes());
    buf.clear();

    // 3:14:15.000001
    Encode::<Postgres>::encode(&time!(3:14:15.000001), &mut buf);
    let expected = 1_000_000i64 * 60 * 60 * 3 + 1_000_000i64 * 60 * 14 + 1_000_000i64 * 15 + 1;
    assert_eq!(buf, expected.to_be_bytes());
    buf.clear();
}

#[test]
fn test_decode_time() {
    let buf = [0u8; 8];
    let time: Time = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(time, time!(0:00));

    // half an hour
    let buf = (1_000_000i64 * 60 * 30).to_be_bytes();
    let time: Time = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(time, time!(0:30));

    // 12:53:05.125305
    let buf = (1_000_000i64 * 60 * 60 * 12 + 1_000_000i64 * 60 * 53 + 1_000_000i64 * 5 + 125305)
        .to_be_bytes();
    let time: Time = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(time, time!(12:53:05.125305));
}

#[test]
fn test_encode_datetime() {
    let mut buf = Vec::new();

    Encode::<Postgres>::encode(&POSTGRES_EPOCH, &mut buf);
    assert_eq!(buf, [0; 8]);
    buf.clear();

    // one hour past epoch
    let date = POSTGRES_EPOCH + 1.hours();
    Encode::<Postgres>::encode(&date, &mut buf);
    assert_eq!(buf, 3_600_000_000i64.to_be_bytes());
    buf.clear();

    // some random date
    let date = PrimitiveDateTime::new(date!(2019 - 12 - 11), time!(11:01:05));
    let expected = (date - POSTGRES_EPOCH).whole_microseconds() as i64;
    Encode::<Postgres>::encode(&date, &mut buf);
    assert_eq!(buf, expected.to_be_bytes());
    buf.clear();
}

#[test]
fn test_decode_datetime() {
    let buf = [0u8; 8];
    let date: PrimitiveDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2000 - 01 - 01), time!(00:00:00))
    );

    let buf = 3_600_000_000i64.to_be_bytes();
    let date: PrimitiveDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2000 - 01 - 01), time!(01:00:00))
    );

    let buf = 629_377_265_000_000i64.to_be_bytes();
    let date: PrimitiveDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2019 - 12 - 11), time!(11:01:05))
    );
}

#[test]
fn test_encode_offsetdatetime() {
    let mut buf = Vec::new();

    Encode::<Postgres>::encode(&POSTGRES_EPOCH.assume_utc(), &mut buf);
    assert_eq!(buf, [0; 8]);
    buf.clear();

    // one hour past epoch in MSK (2 hours before epoch in UTC)
    let date = (POSTGRES_EPOCH + 1.hours()).assume_offset(offset!(+3));
    Encode::<Postgres>::encode(&date, &mut buf);
    assert_eq!(buf, (-7_200_000_000i64).to_be_bytes());
    buf.clear();

    // some random date in MSK
    let date =
        PrimitiveDateTime::new(date!(2019 - 12 - 11), time!(11:01:05)).assume_offset(offset!(+3));
    let expected = (date - POSTGRES_EPOCH.assume_utc()).whole_microseconds() as i64;
    Encode::<Postgres>::encode(&date, &mut buf);
    assert_eq!(buf, expected.to_be_bytes());
    buf.clear();
}

#[test]
fn test_decode_offsetdatetime() {
    let buf = [0u8; 8];
    let date: OffsetDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2000 - 01 - 01), time!(00:00:00)).assume_utc()
    );

    let buf = 3_600_000_000i64.to_be_bytes();
    let date: OffsetDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2000 - 01 - 01), time!(01:00:00)).assume_utc()
    );

    let buf = 629_377_265_000_000i64.to_be_bytes();
    let date: OffsetDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2019 - 12 - 11), time!(11:01:05)).assume_utc()
    );
}

#[test]
fn test_encode_date() {
    let mut buf = Vec::new();

    let date = date!(2000 - 1 - 1);
    Encode::<Postgres>::encode(&date, &mut buf);
    assert_eq!(buf, [0; 4]);
    buf.clear();

    let date = date!(2001 - 1 - 1);
    Encode::<Postgres>::encode(&date, &mut buf);
    // 2000 was a leap year
    assert_eq!(buf, 366i32.to_be_bytes());
    buf.clear();

    let date = date!(2019 - 12 - 11);
    Encode::<Postgres>::encode(&date, &mut buf);
    assert_eq!(buf, 7284i32.to_be_bytes());
    buf.clear();
}

#[test]
fn test_decode_date() {
    let buf = [0; 4];
    let date: Date = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date, date!(2000 - 01 - 01));

    let buf = 366i32.to_be_bytes();
    let date: Date = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date, date!(2001 - 01 - 01));

    let buf = 7284i32.to_be_bytes();
    let date: Date = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date, date!(2019 - 12 - 11));
}
