
Skip to content
Pull requests
Issues
Marketplace
Explore
@XuTuKe
launchbadge /
sqlx

61
4.2k

    400

Code
Issues 260
Pull requests 20
Discussions
Actions
Wiki
Security

    Insights

sqlx/sqlx-core/src/mssql/types/chrono.rs
@danielakhterov
danielakhterov feat: add support for chrono on mssql
Latest commit 416375d on Jul 8, 2020
History
1 contributor
104 lines (85 sloc) 2.86 KB
use std::mem;

use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::mssql::protocol::type_info::{DataType, TypeInfo};
use crate::mssql::{Mssql, MssqlTypeInfo, MssqlValueRef};
use crate::types::Type;

impl Type<Mssql> for NaiveTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::TimeN, 8))
    }
}

impl Type<Mssql> for NaiveDate {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::DateTimeN, 8))
    }
}

impl Type<Mssql> for NaiveDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::DateTimeN, 8))
    }
}

impl<Tz: TimeZone> Type<Mssql> for DateTime<Tz> {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo(TypeInfo::new(DataType::DateTimeOffsetN, 8))
    }
}

impl Encode<'_, Mssql> for NaiveTime {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        todo!()
    }
}

impl<'r> Decode<'r, Mssql> for NaiveTime {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        todo!()
    }
}

impl Encode<'_, Mssql> for NaiveDate {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        todo!()
    }
}

impl<'r> Decode<'r, Mssql> for NaiveDate {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        todo!()
    }
}

impl Encode<'_, Mssql> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        let days_duration = self.date() - NaiveDate::from_ymd(1900, 1, 1);
        let ms_duration = self.time() - NaiveTime::from_hms(0, 0, 0);
        let days = days_duration.num_days() as i32;
        let ms = ms_duration.num_milliseconds() as u32 * 3 / 10;

        buf.extend(&days.to_le_bytes());
        buf.extend_from_slice(&ms.to_le_bytes());

        IsNull::No
    }
}

impl<'r> Decode<'r, Mssql> for NaiveDateTime {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let days = LittleEndian::read_i32(&value.as_bytes()?[0..4]);
        let third_seconds = LittleEndian::read_u32(&value.as_bytes()?[4..8]);
        let ms = third_seconds / 3 * 10;

        let time = NaiveTime::from_hms(0, 0, 0) + Duration::milliseconds(ms.into());
        let date = NaiveDate::from_ymd(1900, 1, 1) + Duration::days(days.into());

        Ok(date.and_time(time))
    }
}

impl<Tz: TimeZone> Encode<'_, Mssql> for DateTime<Tz> {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> IsNull {
        todo!()
    }
}

impl<'r> Decode<'r, Mssql> for DateTime<Local> {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        todo!()
    }
}

impl<'r> Decode<'r, Mssql> for DateTime<Utc> {
    fn decode(value: MssqlValueRef<'r>) -> Result<Self, BoxDynError> {
        todo!()
    }
}

    © 2021 GitHub, Inc.
    Terms
    Privacy
    Security
    Status
    Docs

    Contact GitHub
    Pricing
    API
    Training
    Blog
    About

Loading complete
