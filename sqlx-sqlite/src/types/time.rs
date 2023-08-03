use crate::value::ValueRef;
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    type_info::DataType,
    types::Type,
    Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef,
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
        decode_offset_datetime(value)
    }
}

impl<'r> Decode<'r, Sqlite> for PrimitiveDateTime {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        decode_datetime(value)
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

        let sqlite_time_formats = &[
            fd!("[hour]:[minute]:[second].[subsecond]"),
            fd!("[hour]:[minute]:[second]"),
            fd!("[hour]:[minute]"),
        ];

        for format in sqlite_time_formats {
            if let Ok(dt) = Time::parse(value, &format) {
                return Ok(dt);
            }
        }

        Err(format!("invalid time: {value}").into())
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

    if let Ok(dt) = OffsetDateTime::parse(value, formats::OFFSET_DATE_TIME) {
        return Some(dt);
    }

    if let Some(dt) = decode_datetime_from_text(value) {
        return Some(dt.assume_utc());
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
    let default_format = fd!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");
    if let Ok(dt) = PrimitiveDateTime::parse(value, &default_format) {
        return Some(dt);
    }

    let formats = [
        FormatItem::Compound(formats::PRIMITIVE_DATE_TIME_SPACE_SEPARATED),
        FormatItem::Compound(formats::PRIMITIVE_DATE_TIME_T_SEPARATED),
    ];

    if let Ok(dt) = PrimitiveDateTime::parse(value, &FormatItem::First(&formats)) {
        return Some(dt);
    }

    None
}

mod formats {
    use time::format_description::{modifier, Component::*, FormatItem, FormatItem::*};

    const YEAR: FormatItem<'_> = Component(Year({
        let mut value = modifier::Year::default();
        value.padding = modifier::Padding::Zero;
        value.repr = modifier::YearRepr::Full;
        value.iso_week_based = false;
        value.sign_is_mandatory = false;
        value
    }));

    const MONTH: FormatItem<'_> = Component(Month({
        let mut value = modifier::Month::default();
        value.padding = modifier::Padding::Zero;
        value.repr = modifier::MonthRepr::Numerical;
        value.case_sensitive = true;
        value
    }));

    const DAY: FormatItem<'_> = Component(Day({
        let mut value = modifier::Day::default();
        value.padding = modifier::Padding::Zero;
        value
    }));

    const HOUR: FormatItem<'_> = Component(Hour({
        let mut value = modifier::Hour::default();
        value.padding = modifier::Padding::Zero;
        value.is_12_hour_clock = false;
        value
    }));

    const MINUTE: FormatItem<'_> = Component(Minute({
        let mut value = modifier::Minute::default();
        value.padding = modifier::Padding::Zero;
        value
    }));

    const SECOND: FormatItem<'_> = Component(Second({
        let mut value = modifier::Second::default();
        value.padding = modifier::Padding::Zero;
        value
    }));

    const SUBSECOND: FormatItem<'_> = Component(Subsecond({
        let mut value = modifier::Subsecond::default();
        value.digits = modifier::SubsecondDigits::OneOrMore;
        value
    }));

    const OFFSET_HOUR: FormatItem<'_> = Component(OffsetHour({
        let mut value = modifier::OffsetHour::default();
        value.sign_is_mandatory = true;
        value.padding = modifier::Padding::Zero;
        value
    }));

    const OFFSET_MINUTE: FormatItem<'_> = Component(OffsetMinute({
        let mut value = modifier::OffsetMinute::default();
        value.padding = modifier::Padding::Zero;
        value
    }));

    pub(super) const OFFSET_DATE_TIME: &[FormatItem<'_>] = {
        &[
            YEAR,
            Literal(b"-"),
            MONTH,
            Literal(b"-"),
            DAY,
            Optional(&Literal(b" ")),
            Optional(&Literal(b"T")),
            HOUR,
            Literal(b":"),
            MINUTE,
            Optional(&Literal(b":")),
            Optional(&SECOND),
            Optional(&Literal(b".")),
            Optional(&SUBSECOND),
            Optional(&OFFSET_HOUR),
            Optional(&Literal(b":")),
            Optional(&OFFSET_MINUTE),
        ]
    };

    pub(super) const PRIMITIVE_DATE_TIME_SPACE_SEPARATED: &[FormatItem<'_>] = {
        &[
            YEAR,
            Literal(b"-"),
            MONTH,
            Literal(b"-"),
            DAY,
            Literal(b" "),
            HOUR,
            Literal(b":"),
            MINUTE,
            Optional(&Literal(b":")),
            Optional(&SECOND),
            Optional(&Literal(b".")),
            Optional(&SUBSECOND),
            Optional(&Literal(b"Z")),
        ]
    };

    pub(super) const PRIMITIVE_DATE_TIME_T_SEPARATED: &[FormatItem<'_>] = {
        &[
            YEAR,
            Literal(b"-"),
            MONTH,
            Literal(b"-"),
            DAY,
            Literal(b"T"),
            HOUR,
            Literal(b":"),
            MINUTE,
            Optional(&Literal(b":")),
            Optional(&SECOND),
            Optional(&Literal(b".")),
            Optional(&SUBSECOND),
            Optional(&Literal(b"Z")),
        ]
    };
}
