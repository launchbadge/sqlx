use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    sqlite::{type_info::DataType, Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
    types::Type,
};
use chrono::prelude::*;

impl Type<Sqlite> for NaiveDateTime {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo(DataType::Timestamp)
    }
}

impl Encode<'_, Sqlite> for NaiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> IsNull {
        let text: String = self.format("%F %T%.f").to_string();
        Encode::<Sqlite>::encode(text, buf)
    }
}

impl<'a> Decode<'a, Sqlite> for NaiveDateTime {
    fn decode(value: SqliteValueRef<'a>) -> Result<Self, BoxDynError> {
        let text = Decode::<Sqlite>::decode(value)?;
        decode_from_text(text)
    }
}

fn decode_from_text(text: Option<&str>) -> Result<NaiveDateTime, BoxDynError> {
    if let Some(raw) = text {
        // Loop over common date time patterns, inspired by Diesel
        // https://docs.diesel.rs/src/diesel/sqlite/types/date_and_time/chrono.rs.html#56-97
        let sqlite_datetime_formats = &[
            // Most likely format
            "%F %T%.f",
            // Other formats in order of appearance in docs
            "%F %R",
            "%F %RZ",
            "%F %R%:z",
            "%F %T%.fZ",
            "%F %T%.f%:z",
            "%FT%R",
            "%FT%RZ",
            "%FT%R%:z",
            "%FT%T%.f",
            "%FT%T%.fZ",
            "%FT%T%.f%:z",
        ];

        for format in sqlite_datetime_formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(raw, format) {
                return Ok(dt);
            }
        }

        return Err(err_protocol!("Did not find a matching pattern").into());
    }

    Err(err_protocol!("There was no text value to decode").into())
}
