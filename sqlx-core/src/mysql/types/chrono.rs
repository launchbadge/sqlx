use chrono::{NaiveDateTime, NaiveDate, Timelike, Datelike};
use byteorder::{ByteOrder, LittleEndian};

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::MySql;
use crate::types::HasSqlType;
use crate::mysql::types::{MySqlTypeMetadata};
use crate::mysql::protocol::Type;
use std::convert::TryFrom;

impl HasSqlType<NaiveDateTime> for MySql {
    fn metadata() -> Self::TypeMetadata {
        MySqlTypeMetadata::new(Type::DATETIME)
    }
}

impl Encode<MySql> for NaiveDateTime {
    fn encode(&self, buf: &mut Vec<u8>) {
        // subtract the length byte
        let length = Encode::<MySql>::size_hint(self) - 1;

        buf.push(length as u8);

        encode_date(self.date(), buf);

        if length >= 7 {
            buf.push(self.hour() as u8);
            buf.push(self.minute() as u8);
            buf.push(self.second() as u8);
        }

        if length == 11 {
            buf.extend_from_slice(&self.timestamp_subsec_micros().to_le_bytes());
        }
    }

    fn size_hint(&self) -> usize {
        match (
            self.hour(),
            self.minute(),
            self.second(),
            self.timestamp_subsec_micros(),
        ) {
            // include the length byte
            (0, 0, 0, 0) => 5,
            (_, _, _, 0) => 8,
            (_, _, _, _) => 12,
        }
    }
}

impl Decode<MySql> for NaiveDateTime {
    fn decode(raw: &[u8]) -> Result<Self, DecodeError> {
        let len = raw[0];

        // TODO: Make an error
        assert_ne!(len, 0, "MySQL zero-dates are not supported");

        let date = decode_date(&raw[1..]);

        Ok(if len >= 7 {
            date.and_hms_micro(
                raw[5] as u32,
                raw[6] as u32,
                raw[7] as u32,
                if len == 11 {
                    LittleEndian::read_u32(&raw[8..])
                } else {
                    0
                }
            )
        } else {
            date.and_hms(0, 0, 0)
        })
    }
}

impl HasSqlType<NaiveDate> for MySql {
    fn metadata() -> Self::TypeMetadata {
        MySqlTypeMetadata::new(Type::DATE)
    }
}

impl Encode<MySql> for NaiveDate {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(4);

        encode_date(*self, buf);
    }

    fn size_hint(&self) -> usize {
        5
    }
}

impl Decode<MySql> for NaiveDate {
    fn decode(raw: &[u8]) -> Result<Self, DecodeError> {
        // TODO: Return error
        assert_eq!(raw[0], 4, "expected only 4 bytes");

        Ok(decode_date(&raw[1..]))
    }
}

fn encode_date(date: NaiveDate, buf: &mut Vec<u8>) {
    // MySQL supports years from 1000 - 9999
    let year = u16::try_from(date.year())
        .unwrap_or_else(|_| panic!("NaiveDateTime out of range for Mysql: {}", date));

    buf.extend_from_slice(&year.to_le_bytes());
    buf.push(date.month() as u8);
    buf.push(date.day() as u8);
}

fn decode_date(raw: &[u8]) -> NaiveDate {
    NaiveDate::from_ymd(
        LittleEndian::read_u16(raw) as i32,
        raw[2] as u32,
        raw[3] as u32
    )
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
    let date1 = <NaiveDateTime as Decode<MySql>>::decode(&buf).unwrap();
    assert_eq!(date1.to_string(), "2010-10-17 19:27:30.000001");

    let buf = [7, 218, 7, 10, 17, 19, 27, 30];
    let date2 = <NaiveDateTime as Decode<MySql>>::decode(&buf).unwrap();
    assert_eq!(date2.to_string(), "2010-10-17 19:27:30");

    let buf = [4, 218, 7, 10, 17];
    let date3 = <NaiveDateTime as Decode<MySql>>::decode(&buf).unwrap();
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
    let date = <NaiveDate as Decode<MySql>>::decode(&buf).unwrap();
    assert_eq!(date.to_string(), "2010-10-17");
}
