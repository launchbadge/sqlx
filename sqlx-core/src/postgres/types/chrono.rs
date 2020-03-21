use std::convert::TryInto;
use std::mem;

use byteorder::{NetworkEndian, ReadBytesExt};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::row::PgValue;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::Type;
use crate::Error;

impl Type<Postgres> for NaiveTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIME, "TIME")
    }
}

impl Type<Postgres> for NaiveDate {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::DATE, "DATE")
    }
}

impl Type<Postgres> for NaiveDateTime {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIMESTAMP, "TIMESTAMP")
    }
}

impl<Tz> Type<Postgres> for DateTime<Tz>
where
    Tz: TimeZone,
{
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TIMESTAMPTZ, "TIMESTAMPTZ")
    }
}

impl Type<Postgres> for [NaiveTime] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIME, "TIME[]")
    }
}

impl Type<Postgres> for [NaiveDate] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_DATE, "DATE[]")
    }
}

impl Type<Postgres> for [NaiveDateTime] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIMESTAMP, "TIMESTAMP[]")
    }
}

impl<Tz> Type<Postgres> for [DateTime<Tz>]
where
    Tz: TimeZone,
{
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TIMESTAMPTZ, "TIMESTAMP[]")
    }
}

impl Type<Postgres> for Vec<NaiveTime> {
    fn type_info() -> PgTypeInfo {
        <[NaiveTime] as Type<Postgres>>::type_info()
    }
}

impl Type<Postgres> for Vec<NaiveDate> {
    fn type_info() -> PgTypeInfo {
        <[NaiveDate] as Type<Postgres>>::type_info()
    }
}

impl Type<Postgres> for Vec<NaiveDateTime> {
    fn type_info() -> PgTypeInfo {
        <[NaiveDateTime] as Type<Postgres>>::type_info()
    }
}

impl<Tz> Type<Postgres> for Vec<DateTime<Tz>>
where
    Tz: TimeZone,
{
    fn type_info() -> PgTypeInfo {
        <[NaiveDateTime] as Type<Postgres>>::type_info()
    }
}

impl<'de> Decode<'de, Postgres> for NaiveTime {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => {
                let micros = buf.read_i64::<NetworkEndian>().map_err(Error::decode)?;

                Ok(NaiveTime::from_hms(0, 0, 0) + Duration::microseconds(micros))
            }

            PgValue::Text(s) => NaiveTime::parse_from_str(s, "%H:%M:%S%.f").map_err(Error::decode),
        }
    }
}

impl Encode<Postgres> for NaiveTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        let micros = (*self - NaiveTime::from_hms(0, 0, 0))
            .num_microseconds()
            .expect("shouldn't overflow");

        Encode::<Postgres>::encode(&micros, buf);
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'de> Decode<'de, Postgres> for NaiveDate {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => {
                let days: i32 = buf.read_i32::<NetworkEndian>().map_err(Error::decode)?;

                Ok(NaiveDate::from_ymd(2000, 1, 1) + Duration::days(days as i64))
            }

            PgValue::Text(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(Error::decode),
        }
    }
}

impl Encode<Postgres> for NaiveDate {
    fn encode(&self, buf: &mut Vec<u8>) {
        let days: i32 = self
            .signed_duration_since(NaiveDate::from_ymd(2000, 1, 1))
            .num_days()
            .try_into()
            // TODO: How does Diesel handle this?
            .unwrap_or_else(|_| panic!("NaiveDate out of range for Postgres: {:?}", self));

        Encode::<Postgres>::encode(&days, buf)
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i32>()
    }
}

impl<'de> Decode<'de, Postgres> for NaiveDateTime {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => {
                let micros = buf.read_i64::<NetworkEndian>().map_err(Error::decode)?;

                postgres_epoch()
                    .naive_utc()
                    .checked_add_signed(Duration::microseconds(micros))
                    .ok_or_else(|| {
                        crate::Error::Decode(
                            format!(
                                "Postgres timestamp out of range for NaiveDateTime: {:?}",
                                micros
                            )
                            .into(),
                        )
                    })
            }

            PgValue::Text(s) => {
                NaiveDateTime::parse_from_str(
                    s,
                    if s.contains('+') {
                        // Contains a time-zone specifier
                        // This is given for timestamptz for some reason
                        // Postgres already guarantees this to always be UTC
                        "%Y-%m-%d %H:%M:%S%.f%#z"
                    } else {
                        "%Y-%m-%d %H:%M:%S%.f"
                    },
                )
                .map_err(Error::decode)
            }
        }
    }
}

impl Encode<Postgres> for NaiveDateTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        let micros = self
            .signed_duration_since(postgres_epoch().naive_utc())
            .num_microseconds()
            .unwrap_or_else(|| panic!("NaiveDateTime out of range for Postgres: {:?}", self));

        Encode::<Postgres>::encode(&micros, buf);
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

impl<'de> Decode<'de, Postgres> for DateTime<Utc> {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        let date_time = Decode::<Postgres>::decode(value)?;
        Ok(DateTime::from_utc(date_time, Utc))
    }
}

impl<'de> Decode<'de, Postgres> for DateTime<Local> {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        let date_time = Decode::<Postgres>::decode(value)?;
        Ok(Local.from_utc_datetime(&date_time))
    }
}

impl<Tz: TimeZone> Encode<Postgres> for DateTime<Tz>
where
    Tz::Offset: Copy,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        Encode::<Postgres>::encode(&self.naive_utc(), buf);
    }

    fn size_hint(&self) -> usize {
        mem::size_of::<i64>()
    }
}

fn postgres_epoch() -> DateTime<Utc> {
    Utc.ymd(2000, 1, 1).and_hms(0, 0, 0)
}

#[test]
fn test_encode_datetime() {
    let mut buf = Vec::new();

    let date = postgres_epoch();
    Encode::<Postgres>::encode(&date, &mut buf);
    assert_eq!(buf, [0; 8]);
    buf.clear();

    // one hour past epoch
    let date2 = postgres_epoch() + Duration::hours(1);
    Encode::<Postgres>::encode(&date2, &mut buf);
    assert_eq!(buf, 3_600_000_000i64.to_be_bytes());
    buf.clear();

    // some random date
    let date3: NaiveDateTime = "2019-12-11T11:01:05".parse().unwrap();
    let expected = dbg!((date3 - postgres_epoch().naive_utc())
        .num_microseconds()
        .unwrap());
    Encode::<Postgres>::encode(&date3, &mut buf);
    assert_eq!(buf, expected.to_be_bytes());
    buf.clear();
}

#[test]
fn test_decode_datetime() {
    let buf = [0u8; 8];
    let date: NaiveDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date.to_string(), "2000-01-01 00:00:00");

    let buf = 3_600_000_000i64.to_be_bytes();
    let date: NaiveDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date.to_string(), "2000-01-01 01:00:00");

    let buf = 629_377_265_000_000i64.to_be_bytes();
    let date: NaiveDateTime = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date.to_string(), "2019-12-11 11:01:05");
}

#[test]
fn test_encode_date() {
    let mut buf = Vec::new();

    let date = NaiveDate::from_ymd(2000, 1, 1);
    Encode::<Postgres>::encode(&date, &mut buf);
    assert_eq!(buf, [0; 4]);
    buf.clear();

    let date2 = NaiveDate::from_ymd(2001, 1, 1);
    Encode::<Postgres>::encode(&date2, &mut buf);
    // 2000 was a leap year
    assert_eq!(buf, 366i32.to_be_bytes());
    buf.clear();

    let date3 = NaiveDate::from_ymd(2019, 12, 11);
    Encode::<Postgres>::encode(&date3, &mut buf);
    assert_eq!(buf, 7284i32.to_be_bytes());
    buf.clear();
}

#[test]
fn test_decode_date() {
    let buf = [0; 4];
    let date: NaiveDate = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date.to_string(), "2000-01-01");

    let buf = 366i32.to_be_bytes();
    let date: NaiveDate = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date.to_string(), "2001-01-01");

    let buf = 7284i32.to_be_bytes();
    let date: NaiveDate = Decode::<Postgres>::decode(Some(PgValue::Binary(&buf))).unwrap();
    assert_eq!(date.to_string(), "2019-12-11");
}
