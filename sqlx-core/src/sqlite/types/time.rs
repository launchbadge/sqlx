use crate::value::ValueRef;
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    sqlite::{type_info::DataType, Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
    types::Type,
};
use time::format_description::{well_known::Rfc3339, FormatItem};
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

        let second = FormatItem::Optional(&FormatItem::Compound(fd!(":[second]")));
        let subsecond = FormatItem::Optional(&FormatItem::Compound(fd!(".[subsecond]")));
        let full_description = [fd!("[hour]:[minute]"), &[second], &[subsecond]].concat();
        let format = [FormatItem::Compound(&full_description[..])];

        let result = Time::parse(value, &FormatItem::First(&format));
        if let Ok(time) = result {
            return Ok(time);
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

    // Support for other SQLite supported formats not served by Rfc3339
    let ymd = fd!("[year]-[month]-[day]");
    let hm = fd!("[hour]:[minute]");
    let t_variant_base = [ymd, &[FormatItem::Literal(b"T")], hm].concat();
    let space_variant_base = [ymd, &[FormatItem::Literal(b" ")], hm].concat();

    let optionals = [
        FormatItem::Optional(&FormatItem::Compound(fd!(":[second]"))),
        FormatItem::Optional(&FormatItem::Compound(fd!(".[subsecond]"))),
        FormatItem::Optional(&FormatItem::Compound(fd!(
            "[offset_hour sign:mandatory]:[offset_minute]"
        ))),
    ];

    let t_variant_full = [&t_variant_base[..], &optionals[..]].concat();
    let space_variant_full = [&space_variant_base[..], &optionals[..]].concat();

    let formats = [
        FormatItem::Compound(&space_variant_full),
        FormatItem::Compound(&t_variant_full),
    ];

    if let Ok(dt) = OffsetDateTime::parse(value, &FormatItem::First(&formats)) {
        return Some(dt);
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
    let ymd = fd!("[year]-[month]-[day]");
    let hm = fd!("[hour]:[minute]");
    let t_variant_base = [ymd, &[FormatItem::Literal(b"T")], hm].concat();
    let space_variant_base = [ymd, &[FormatItem::Literal(b" ")], hm].concat();

    let optionals = [
        FormatItem::Optional(&FormatItem::Compound(fd!(":[second]"))),
        FormatItem::Optional(&FormatItem::Compound(fd!(".[subsecond]"))),
        FormatItem::Optional(&FormatItem::Literal(b"Z")),
    ];

    let t_variant_full = [&t_variant_base[..], &optionals[..]].concat();
    let space_variant_full = [&space_variant_base[..], &optionals[..]].concat();

    let formats = [
        FormatItem::Compound(&space_variant_full),
        FormatItem::Compound(&t_variant_full),
    ];

    if let Ok(dt) = PrimitiveDateTime::parse(value, &FormatItem::First(&formats)) {
        return Some(dt);
    }

    None
}
