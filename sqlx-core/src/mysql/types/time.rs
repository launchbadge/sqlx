use std::borrow::Cow;
use std::convert::TryFrom;
use std::convert::TryInto;

use byteorder::{ByteOrder, LittleEndian};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::io::{Buf, BufMut};
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::{MySql, MySqlValue};
use crate::types::Type;

impl Type<MySql> for OffsetDateTime {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::TIMESTAMP)
    }
}

impl Encode<MySql> for OffsetDateTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        let utc_dt = self.to_offset(UtcOffset::UTC);
        let primitive_dt = PrimitiveDateTime::new(utc_dt.date(), utc_dt.time());

        Encode::<MySql>::encode(&primitive_dt, buf);
    }
}

impl<'de> Decode<'de, MySql> for OffsetDateTime {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        let primitive: PrimitiveDateTime = Decode::<MySql>::decode(value)?;

        Ok(primitive.assume_utc())
    }
}

impl Type<MySql> for Time {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::TIME)
    }
}

impl Encode<MySql> for Time {
    fn encode(&self, buf: &mut Vec<u8>) {
        let len = Encode::<MySql>::size_hint(self) - 1;
        buf.push(len as u8);

        // Time is not negative
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

impl<'de> Decode<'de, MySql> for Time {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
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
                let s = from_utf8(buf).map_err(crate::Error::decode)?;

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

impl Type<MySql> for Date {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::DATE)
    }
}

impl Encode<MySql> for Date {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(4);

        encode_date(self, buf);
    }

    fn size_hint(&self) -> usize {
        5
    }
}

impl<'de> Decode<'de, MySql> for Date {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(buf) => decode_date(&buf[1..]),
            MySqlValue::Text(buf) => {
                let s = from_utf8(buf).map_err(crate::Error::decode)?;
                Date::parse(s, "%Y-%m-%d").map_err(crate::Error::decode)
            }
        }
    }
}

impl Type<MySql> for PrimitiveDateTime {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::DATETIME)
    }
}

impl Encode<MySql> for PrimitiveDateTime {
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
        match (self.hour(), self.minute(), self.second(), self.nanosecond()) {
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

impl<'de> Decode<'de, MySql> for PrimitiveDateTime {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(buf) => {
                let len = buf[0];
                let date = decode_date(&buf[1..])?;

                let dt = if len > 4 {
                    date.with_time(decode_time(len - 4, &buf[5..])?)
                } else {
                    date.midnight()
                };

                Ok(dt)
            }

            MySqlValue::Text(buf) => {
                let s = from_utf8(buf).map_err(crate::Error::decode)?;

                // If there are less than 9 digits after the decimal point
                // We need to zero-pad
                // TODO: Ask [time] to add a parse % for less-than-fixed-9 nanos

                let s = if s.len() < 31 {
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

fn encode_date(date: &Date, buf: &mut Vec<u8>) {
    // MySQL supports years from 1000 - 9999
    let year = u16::try_from(date.year())
        .unwrap_or_else(|_| panic!("Date out of range for Mysql: {}", date));

    buf.extend_from_slice(&year.to_le_bytes());
    buf.push(date.month());
    buf.push(date.day());
}

fn decode_date(buf: &[u8]) -> crate::Result<MySql, Date> {
    Date::try_from_ymd(
        LittleEndian::read_u16(buf) as i32,
        buf[2] as u8,
        buf[3] as u8,
    )
    .map_err(|e| decode_err!("Error while decoding Date: {}", e))
}

fn encode_time(time: &Time, include_micros: bool, buf: &mut Vec<u8>) {
    buf.push(time.hour());
    buf.push(time.minute());
    buf.push(time.second());

    if include_micros {
        buf.put_u32::<LittleEndian>((time.nanosecond() / 1000) as u32);
    }
}

fn decode_time(len: u8, mut buf: &[u8]) -> crate::Result<MySql, Time> {
    let hour = buf.get_u8()?;
    let minute = buf.get_u8()?;
    let seconds = buf.get_u8()?;

    let micros = if len > 3 {
        // microseconds : int<EOF>
        buf.get_uint::<LittleEndian>(buf.len())?
    } else {
        0
    };

    Time::try_from_hms_micro(hour, minute, seconds, micros as u32)
        .map_err(|e| decode_err!("Time out of range for MySQL: {}", e))
}

use std::str::from_utf8;
#[cfg(test)]
use time::{date, time};

#[test]
fn test_encode_date_time() {
    let mut buf = Vec::new();

    // test values from https://dev.mysql.com/doc/internals/en/binary-protocol-value.html
    let date = PrimitiveDateTime::new(date!(2010 - 10 - 17), time!(19:27:30.000001));
    Encode::<MySql>::encode(&date, &mut buf);
    assert_eq!(*buf, [11, 218, 7, 10, 17, 19, 27, 30, 1, 0, 0, 0]);

    buf.clear();

    let date = PrimitiveDateTime::new(date!(2010 - 10 - 17), time!(19:27:30));
    Encode::<MySql>::encode(&date, &mut buf);
    assert_eq!(*buf, [7, 218, 7, 10, 17, 19, 27, 30]);

    buf.clear();

    let date = PrimitiveDateTime::new(date!(2010 - 10 - 17), time!(00:00:00));
    Encode::<MySql>::encode(&date, &mut buf);
    assert_eq!(*buf, [4, 218, 7, 10, 17]);
}

#[test]
fn test_decode_date_time() {
    // test values from https://dev.mysql.com/doc/internals/en/binary-protocol-value.html
    let buf = [11, 218, 7, 10, 17, 19, 27, 30, 1, 0, 0, 0];
    let date1 =
        <PrimitiveDateTime as Decode<MySql>>::decode(Some(MySqlValue::Binary(&buf))).unwrap();
    assert_eq!(date1.to_string(), "2010-10-17 19:27:30.000001");

    let buf = [7, 218, 7, 10, 17, 19, 27, 30];
    let date2 =
        <PrimitiveDateTime as Decode<MySql>>::decode(Some(MySqlValue::Binary(&buf))).unwrap();
    assert_eq!(date2.to_string(), "2010-10-17 19:27:30");

    let buf = [4, 218, 7, 10, 17];
    let date3 =
        <PrimitiveDateTime as Decode<MySql>>::decode(Some(MySqlValue::Binary(&buf))).unwrap();
    assert_eq!(date3.to_string(), "2010-10-17 0:00");
}

#[test]
fn test_encode_date() {
    let mut buf = Vec::new();
    let date: Date = date!(2010 - 10 - 17);
    Encode::<MySql>::encode(&date, &mut buf);
    assert_eq!(*buf, [4, 218, 7, 10, 17]);
}

#[test]
fn test_decode_date() {
    let buf = [4, 218, 7, 10, 17];
    let date = <Date as Decode<MySql>>::decode(Some(MySqlValue::Binary(&buf))).unwrap();
    assert_eq!(date, date!(2010 - 10 - 17));
}
