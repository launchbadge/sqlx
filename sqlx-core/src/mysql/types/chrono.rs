use std::convert::{TryFrom, TryInto};

use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::io::{Buf, BufMut};
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::{MySql, MySqlValue};
use crate::types::Type;
use crate::Error;
use bitflags::_core::str::from_utf8;

impl Type<MySql> for DateTime<Utc> {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::TIMESTAMP)
    }
}

impl Encode<MySql> for DateTime<Utc> {
    fn encode(&self, buf: &mut Vec<u8>) {
        Encode::<MySql>::encode(&self.naive_utc(), buf);
    }
}

impl<'de> Decode<'de, MySql> for DateTime<Utc> {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        let naive: NaiveDateTime = Decode::<MySql>::decode(value)?;

        Ok(DateTime::from_utc(naive, Utc))
    }
}

impl Type<MySql> for NaiveTime {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::TIME)
    }
}

impl Encode<MySql> for NaiveTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        let len = Encode::<MySql>::size_hint(self) - 1;
        buf.push(len as u8);

        // NaiveTime is not negative
        buf.push(0);

        // "date on 4 bytes little-endian format" (?)
        // https://mariadb.com/kb/en/resultset-row/#teimstamp-binary-encoding
        buf.advance(4);

        encode_time(self, len > 9, buf);
    }

    fn size_hint(&self) -> usize {
        if self.nanosecond() == 0 {
            // if micro_seconds is 0, length is 8 and micro_seconds is not sent
            9
        } else {
            // otherwise length is 12
            13
        }
    }
}

impl<'de> Decode<'de, MySql> for NaiveTime {
    fn decode(buf: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match buf.try_into()? {
            MySqlValue::Binary(mut buf) => {
                // data length, expecting 8 or 12 (fractional seconds)
                let len = buf.get_u8()?;

                // is negative : int<1>
                let is_negative = buf.get_u8()?;
                assert_eq!(is_negative, 0, "Negative dates/times are not supported");

                // "date on 4 bytes little-endian format" (?)
                // https://mariadb.com/kb/en/resultset-row/#timestamp-binary-encoding
                buf.advance(4);

                decode_time(len - 5, buf)
            }

            MySqlValue::Text(buf) => {
                let s = from_utf8(buf).map_err(Error::decode)?;
                NaiveTime::parse_from_str(s, "%H:%M:%S%.f").map_err(Error::decode)
            }
        }
    }
}

impl Type<MySql> for NaiveDate {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::DATE)
    }
}

impl Encode<MySql> for NaiveDate {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(4);

        encode_date(self, buf);
    }

    fn size_hint(&self) -> usize {
        5
    }
}

impl<'de> Decode<'de, MySql> for NaiveDate {
    fn decode(buf: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match buf.try_into()? {
            MySqlValue::Binary(buf) => Ok(decode_date(&buf[1..])),

            MySqlValue::Text(buf) => {
                let s = from_utf8(buf).map_err(Error::decode)?;
                NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(Error::decode)
            }
        }
    }
}

impl Type<MySql> for NaiveDateTime {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::DATETIME)
    }
}

impl Encode<MySql> for NaiveDateTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        let len = Encode::<MySql>::size_hint(self) - 1;
        buf.push(len as u8);

        encode_date(&self.date(), buf);

        if len > 4 {
            encode_time(&self.time(), len > 8, buf);
        }
    }

    fn size_hint(&self) -> usize {
        // to save space the packet can be compressed:
        match (
            self.hour(),
            self.minute(),
            self.second(),
            self.timestamp_subsec_nanos(),
        ) {
            // if hour, minutes, seconds and micro_seconds are all 0,
            // length is 4 and no other field is sent
            (0, 0, 0, 0) => 5,

            // if micro_seconds is 0, length is 7
            // and micro_seconds is not sent
            (_, _, _, 0) => 8,

            // otherwise length is 11
            (_, _, _, _) => 12,
        }
    }
}

impl<'de> Decode<'de, MySql> for NaiveDateTime {
    fn decode(buf: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match buf.try_into()? {
            MySqlValue::Binary(buf) => {
                let len = buf[0];
                let date = decode_date(&buf[1..]);

                let dt = if len > 4 {
                    date.and_time(decode_time(len - 4, &buf[5..])?)
                } else {
                    date.and_hms(0, 0, 0)
                };

                Ok(dt)
            }

            MySqlValue::Text(buf) => {
                let s = from_utf8(buf).map_err(Error::decode)?;
                NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").map_err(Error::decode)
            }
        }
    }
}

fn encode_date(date: &NaiveDate, buf: &mut Vec<u8>) {
    // MySQL supports years from 1000 - 9999
    let year = u16::try_from(date.year())
        .unwrap_or_else(|_| panic!("NaiveDateTime out of range for Mysql: {}", date));

    buf.extend_from_slice(&year.to_le_bytes());
    buf.push(date.month() as u8);
    buf.push(date.day() as u8);
}

fn decode_date(buf: &[u8]) -> NaiveDate {
    NaiveDate::from_ymd(
        LittleEndian::read_u16(buf) as i32,
        buf[2] as u32,
        buf[3] as u32,
    )
}

fn encode_time(time: &NaiveTime, include_micros: bool, buf: &mut Vec<u8>) {
    buf.push(time.hour() as u8);
    buf.push(time.minute() as u8);
    buf.push(time.second() as u8);

    if include_micros {
        buf.put_u32::<LittleEndian>((time.nanosecond() / 1000) as u32);
    }
}

fn decode_time(len: u8, mut buf: &[u8]) -> crate::Result<MySql, NaiveTime> {
    let hour = buf.get_u8()?;
    let minute = buf.get_u8()?;
    let seconds = buf.get_u8()?;

    let micros = if len > 3 {
        // microseconds : int<EOF>
        buf.get_uint::<LittleEndian>(buf.len())?
    } else {
        0
    };

    Ok(NaiveTime::from_hms_micro(
        hour as u32,
        minute as u32,
        seconds as u32,
        micros as u32,
    ))
}

#[test]
fn test_encode_date_time() {
    let mut buf = Vec::new();

    // test values from https://dev.mysql.com/doc/internals/en/binary-protocol-value.html
    let date1: NaiveDateTime = "2010-10-17T19:27:30.000001".parse().unwrap();
    Encode::<MySql>::encode(&date1, &mut buf);
    assert_eq!(*buf, [11, 218, 7, 10, 17, 19, 27, 30, 1, 0, 0, 0]);

    buf.clear();

    let date2: NaiveDateTime = "2010-10-17T19:27:30".parse().unwrap();
    Encode::<MySql>::encode(&date2, &mut buf);
    assert_eq!(*buf, [7, 218, 7, 10, 17, 19, 27, 30]);

    buf.clear();

    let date3: NaiveDateTime = "2010-10-17T00:00:00".parse().unwrap();
    Encode::<MySql>::encode(&date3, &mut buf);
    assert_eq!(*buf, [4, 218, 7, 10, 17]);
}

#[test]
fn test_decode_date_time() {
    // test values from https://dev.mysql.com/doc/internals/en/binary-protocol-value.html
    let buf = [11, 218, 7, 10, 17, 19, 27, 30, 1, 0, 0, 0];
    let date1 = <NaiveDateTime as Decode<MySql>>::decode(Some(MySqlValue::Binary(&buf))).unwrap();
    assert_eq!(date1.to_string(), "2010-10-17 19:27:30.000001");

    let buf = [7, 218, 7, 10, 17, 19, 27, 30];
    let date2 = <NaiveDateTime as Decode<MySql>>::decode(Some(MySqlValue::Binary(&buf))).unwrap();
    assert_eq!(date2.to_string(), "2010-10-17 19:27:30");

    let buf = [4, 218, 7, 10, 17];
    let date3 = <NaiveDateTime as Decode<MySql>>::decode(Some(MySqlValue::Binary(&buf))).unwrap();
    assert_eq!(date3.to_string(), "2010-10-17 00:00:00");
}

#[test]
fn test_encode_date() {
    let mut buf = Vec::new();
    let date: NaiveDate = "2010-10-17".parse().unwrap();
    Encode::<MySql>::encode(&date, &mut buf);
    assert_eq!(*buf, [4, 218, 7, 10, 17]);
}

#[test]
fn test_decode_date() {
    let buf = [4, 218, 7, 10, 17];
    let date = <NaiveDate as Decode<MySql>>::decode(Some(MySqlValue::Binary(&buf))).unwrap();
    assert_eq!(date.to_string(), "2010-10-17");
}
