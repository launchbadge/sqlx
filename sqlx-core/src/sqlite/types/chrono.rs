use crate::{
    decode::Decode,
    encode::Encode,
    sqlite::{
        type_info::{SqliteType, SqliteTypeAffinity},
        Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValue,
    },
    types::Type,
    Result,
};
use chrono::NaiveDateTime;

impl Type<Sqlite> for NaiveDateTime {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Timestamp, SqliteTypeAffinity::Text)
    }
}

impl Encode<Sqlite> for NaiveDateTime {
    fn encode(&self, buf: &mut Vec<SqliteArgumentValue>) {
        buf.push(SqliteArgumentValue::Text(
            self.format("%F %T%.f").to_string(),
        ))
    }
}

impl<'a> Decode<'a, Sqlite> for NaiveDateTime {
    fn decode(value: SqliteValue) -> Result<Self> {
        let the_type = value.r#type();

        match the_type {
            Some(SqliteType::Text) => return decode_from_text(value.text()),
            _ => {}
        }

        Err(protocol_err!("Unexpected affinity for data: {:?} in value", the_type).into())
    }
}

fn decode_from_text(text: Option<&str>) -> Result<NaiveDateTime> {
    if let Some(raw) = text {
        // Loop over common date time partterns, inspired by Diesel
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

        return Err(protocol_err!("Did not find a matching pattern").into());
    }

    Err(protocol_err!("There was not text value to decode").into())
}
