use std::convert::TryInto;
use std::mem;

use time::{date, offset, Date, NumericalDuration, OffsetDateTime, PrimitiveDateTime, Time};

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::HasSqlType;

const POSTGRES_EPOCH: PrimitiveDateTime = date!(2000 - 1 - 1).midnight();

impl HasSqlType<Time> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIME)
    }
}

impl HasSqlType<Date> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::DATE)
    }
}

impl HasSqlType<PrimitiveDateTime> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIMESTAMP)
    }
}

impl HasSqlType<OffsetDateTime> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIMESTAMPTZ)
    }
}

impl HasSqlType<[Time]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIME)
    }
}

impl HasSqlType<[Date]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_DATE)
    }
}

impl HasSqlType<[PrimitiveDateTime]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIMESTAMP)
    }
}

impl HasSqlType<[OffsetDateTime]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIMESTAMPTZ)
    }
}

fn microseconds_since_midnight(time: Time) -> i64 {
    time.hour() as i64 * 60 * 60 * 1_000_000
        + time.minute() as i64 * 60 * 1_000_000
        + time.second() as i64 * 1_000_000
        + time.microsecond() as i64
}

fn from_microseconds_since_midnight(mut microsecond: u64) -> Result<Time, DecodeError> {
    #![allow(clippy::cast_possible_truncation)]

    microsecond %= 86_400 * 1_000_000;

    Time::try_from_hms_micro(
        (microsecond / 1_000_000 / 60 / 60) as u8,
        (microsecond / 1_000_000 / 60 % 60) as u8,
        (microsecond / 1_000_000 % 60) as u8,
        (microsecond % 1_000_000) as u32,
    )
    .map_err(|e| DecodeError::Message(Box::new(format!("Time out of range for Postgres: {}", e))))
}

impl Decode<Postgres> for Time {
    fn decode(raw: &[u8]) -> Result<Self, DecodeError> {
        let micros: i64 = Decode::<Postgres>::decode(raw)?;

        from_microseconds_since_midnight(micros as u64)
    }
}

impl Encode<Postgres> for Time {
    fn encode(&self, buf: &mut Vec<u8>) {
        let micros = microseconds_since_midnight(*self);

        Encode::<Postgres>::encode(&(micros), buf);
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<u64>()
    }
}

impl Decode<Postgres> for Date {
    fn decode(raw: &[u8]) -> Result<Self, DecodeError> {
        let n: i32 = Decode::<Postgres>::decode(raw)?;

        Ok(date!(2000 - 1 - 1) + (n as i64).days())
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

impl Decode<Postgres> for PrimitiveDateTime {
    fn decode(raw: &[u8]) -> Result<Self, DecodeError> {
        let n: i64 = Decode::<Postgres>::decode(raw)?;

        Ok(POSTGRES_EPOCH + n.microseconds())
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

impl Decode<Postgres> for OffsetDateTime {
    fn decode(raw: &[u8]) -> Result<Self, DecodeError> {
        let date_time: PrimitiveDateTime = Decode::<Postgres>::decode(raw)?;
        Ok(date_time.assume_utc())
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
    let expected =
        1_000_000i64 * 60 * 60 * 3 +
        1_000_000i64 * 60 * 14 +
        1_000_000i64 * 15 +
        1
    ;
    assert_eq!(buf, expected.to_be_bytes());
    buf.clear();
}


#[test]
fn test_decode_time() {
    let buf = [0u8; 8];
    let time: Time = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(
        time,
        time!(0:00),
    );

    // half an hour
    let buf = (1_000_000i64 * 60 * 30).to_be_bytes();
    let time: Time = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(
        time,
        time!(0:30),
    );

    // 12:53:05.125305
    let buf = (
        1_000_000i64 * 60 * 60 * 12 +
        1_000_000i64 * 60 * 53 +
        1_000_000i64 * 5 +
        125305
    ).to_be_bytes();
    let time: Time = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(
        time,
        time!(12:53:05.125305),
    );
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
    let date: PrimitiveDateTime = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2000 - 01 - 01), time!(00:00:00))
    );

    let buf = 3_600_000_000i64.to_be_bytes();
    let date: PrimitiveDateTime = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2000 - 01 - 01), time!(01:00:00))
    );

    let buf = 629_377_265_000_000i64.to_be_bytes();
    let date: PrimitiveDateTime = Decode::<Postgres>::decode(&buf).unwrap();
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
    let date: OffsetDateTime = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2000 - 01 - 01), time!(00:00:00)).assume_utc()
    );

    let buf = 3_600_000_000i64.to_be_bytes();
    let date: OffsetDateTime = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(
        date,
        PrimitiveDateTime::new(date!(2000 - 01 - 01), time!(01:00:00)).assume_utc()
    );

    let buf = 629_377_265_000_000i64.to_be_bytes();
    let date: OffsetDateTime = Decode::<Postgres>::decode(&buf).unwrap();
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
    let date: Date = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(date, date!(2000 - 01 - 01));

    let buf = 366i32.to_be_bytes();
    let date: Date = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(date, date!(2001 - 01 - 01));

    let buf = 7284i32.to_be_bytes();
    let date: Date = Decode::<Postgres>::decode(&buf).unwrap();
    assert_eq!(date, date!(2019 - 12 - 11));
}
