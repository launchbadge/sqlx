use crate::value::ValueRef;
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    sqlite::{type_info::DataType, Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
    types::Type,
};
use time::format_description::well_known::Rfc3339;
use time::macros::format_description as fd;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

impl Type<Sqlite> for OffsetDateTime {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Datetime)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <PrimitiveDateTime as Type<Sqlite>>::compatible(ty)
    }
}

impl Type<Sqlite> for PrimitiveDateTime {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Datetime)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(
            ty.0,
            DataType::Datetime | DataType::Text | DataType::Int64 | DataType::Int
        )
    }
}

impl Type<Sqlite> for Date {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Date)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Date | DataType::Text)
    }
}

impl Type<Sqlite> for Time {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Time)
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        matches!(ty.0, DataType::Time | DataType::Text)
    }
}

impl Encode<'_, Sqlite> for OffsetDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        Encode::<Sqlite>::encode(self.format(&Rfc3339).unwrap(), buf)
    }
}

impl Encode<'_, Sqlite> for PrimitiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        let format = fd!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");
        Encode::<Sqlite>::encode(self.format(&format).unwrap(), buf)
    }
}

impl Encode<'_, Sqlite> for Date {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        let format = fd!("[year]-[month]-[day]");
        Encode::<Sqlite>::encode(self.format(&format).unwrap(), buf)
    }
}

impl Encode<'_, Sqlite> for Time {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        let format = fd!("[hour]:[minute]:[second].[subsecond]");
        Encode::<Sqlite>::encode(self.format(&format).unwrap(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for OffsetDateTime {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(decode_offset_datetime(value)?)
    }
}

impl<'r> Decode<'r, Sqlite> for PrimitiveDateTime {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(decode_datetime(value)?)
    }
}

impl<'r> Decode<'r, Sqlite> for Date {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(Date::parse(value.text()?, &fd!("[year]-[month]-[day]"))?)
    }
}

impl<'r> Decode<'r, Sqlite> for Time {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let value = value.text()?;

        // Loop over common time patterns
        let sqlite_time_formats = &[
            // Chosen first since it matches Sqlite time() function
            fd!("[hour]:[minute]:[second]"),
            fd!("[hour]:[minute]:[second].[subsecond]"),
            fd!("[hour]:[minute]"),
        ];

        for format in sqlite_time_formats {
            if let Ok(dt) = Time::parse(value, &format) {
                return Ok(dt);
            }
        }

        Err(format!("invalid time: {}", value).into())
    }
}

fn decode_offset_datetime(value: SqliteValueRef<'_>) -> Result<OffsetDateTime, BoxDynError> {
    let dt = match value.type_info().0 {
        DataType::Text => decode_offset_datetime_from_text(value.text()?),
        DataType::Int | DataType::Int64 => {
            Some(OffsetDateTime::from_unix_timestamp(value.int64())?)
        }

        _ => None,
    };

    if let Some(dt) = dt {
        Ok(dt)
    } else {
        Err(format!("invalid offset datetime: {}", value.text()?).into())
    }
}

fn decode_offset_datetime_from_text(value: &str) -> Option<OffsetDateTime> {
    if let Ok(dt) = OffsetDateTime::parse(value, &Rfc3339) {
        return Some(dt);
    }

    // Loop over common date time patterns
    #[rustfmt::skip] // don't like how rustfmt mangles the comments
    let sqlite_datetime_formats = &[
        fd!("[year]-[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"),
        fd!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"),
        fd!("[year]-[month]-[day] [hour]:[minute][offset_hour sign:mandatory]:[offset_minute]"),
        // Further "T" variants with seconds are covered by parsing with Rfc3339 above
        fd!("[year]-[month]-[day]T[hour]:[minute][offset_hour sign:mandatory]:[offset_minute]"),
    ];

    for format in sqlite_datetime_formats {
        if let Ok(dt) = OffsetDateTime::parse(value, &format) {
            return Some(dt);
        }
    }

    None
}

fn decode_datetime(value: SqliteValueRef<'_>) -> Result<PrimitiveDateTime, BoxDynError> {
    let dt = match value.type_info().0 {
        DataType::Text => decode_datetime_from_text(value.text()?),
        DataType::Int | DataType::Int64 => {
            let parsed = OffsetDateTime::from_unix_timestamp(value.int64()).unwrap();
            Some(PrimitiveDateTime::new(parsed.date(), parsed.time()))
        }

        _ => None,
    };

    if let Some(dt) = dt {
        Ok(dt)
    } else {
        Err(format!("invalid datetime: {}", value.text()?).into())
    }
}

fn decode_datetime_from_text(value: &str) -> Option<PrimitiveDateTime> {
    // Loop over common date time patterns
    #[rustfmt::skip] // don't like how rustfmt mangles the comments
    let sqlite_datetime_formats = &[
        // Chosen first because it matches Sqlite's datetime() function
        fd!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        fd!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"),
        fd!("[year]-[month]-[day] [hour]:[minute]"),
        fd!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
        fd!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]"),
        fd!("[year]-[month]-[day]T[hour]:[minute]"),
        fd!("[year]-[month]-[day] [hour]:[minute]:[second]Z"),
        fd!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]Z"),
        fd!("[year]-[month]-[day] [hour]:[minute]Z"),
        fd!("[year]-[month]-[day]T[hour]:[minute]:[second]Z"),
        fd!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]Z"),
        fd!("[year]-[month]-[day]T[hour]:[minute]Z"),
    ];

    for format in sqlite_datetime_formats {
        if let Ok(dt) = PrimitiveDateTime::parse(value, &format) {
            return Some(dt);
        }
    }

    None
}
