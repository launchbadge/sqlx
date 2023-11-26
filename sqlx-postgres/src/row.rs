use crate::column::ColumnIndex;
use crate::error::Error;
use crate::message::DataRow;
use crate::statement::PgStatementMetadata;
use crate::type_info::PgType;
use crate::types::*;
use crate::value::PgValueFormat;
use crate::{PgColumn, PgValueRef, Postgres};
use std::fmt::{Debug, DebugMap};
use std::sync::Arc;

use sqlx_core::decode::Decode;
use sqlx_core::ext::ustr::UStr;
pub(crate) use sqlx_core::row::Row;

/// Implementation of [`Row`] for PostgreSQL.
pub struct PgRow {
    pub(crate) data: DataRow,
    pub(crate) format: PgValueFormat,
    pub(crate) metadata: Arc<PgStatementMetadata>,
}

impl Row for PgRow {
    type Database = Postgres;

    fn columns(&self) -> &[PgColumn] {
        &self.metadata.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<PgValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let column = &self.metadata.columns[index];
        let value = self.data.get(index);

        Ok(PgValueRef {
            format: self.format,
            row: Some(&self.data.storage),
            type_info: column.type_info.clone(),
            value,
        })
    }
}

impl ColumnIndex<PgRow> for &'_ str {
    fn index(&self, row: &PgRow) -> Result<usize, Error> {
        row.metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

impl Debug for PgRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PgRow: ")?;

        let mut debug_map = f.debug_map();
        for (index, column) in self.columns().iter().enumerate() {
            add_debug_entry(&mut debug_map, self, index, column);
        }

        debug_map.finish()
    }
}

macro_rules! debug_types {
    { $($enum:ident::$variant:ident => $type:ty),* } => {
        fn add_debug_entry(
            debug_map: &mut DebugMap<'_, '_>,
            row: &PgRow,
            index: usize,
            column: &PgColumn) {
                let name = &column.name;
                match row.try_get_raw(index) {
                    Ok(value) => {
                        match column.type_info.0 {
                            $(
                                $enum::$variant => add_decoded_entry::<$type>(debug_map, value, name),
                            )*
                            _ => add_raw_entry(debug_map, value, name)
                        }
                    }
                    _ => {
                        debug_map.entry(name, &"NOT FOUND");
                    }
                }

        }
    }
}

fn add_decoded_entry<'r, T: Decode<'r, Postgres> + Debug>(
    debug_map: &mut DebugMap<'_, '_>,
    value: PgValueRef<'r>,
    name: &UStr,
) {
    match T::decode(value.clone()) {
        Ok(decoded_value) => {
            debug_map.entry(name, &decoded_value);
        }
        _ => {
            add_raw_entry(debug_map, value, name);
        }
    };
}

fn add_raw_entry(debug_map: &mut DebugMap<'_, '_>, value: PgValueRef, name: &UStr) {
    match value.format {
        PgValueFormat::Text => debug_map.entry(name, &value.as_str().unwrap_or_default()),
        PgValueFormat::Binary => debug_map.entry(name, &value.as_bytes().unwrap_or_default()),
    };
}

debug_types! {
    PgType::Money => PgMoney,
    PgType::MoneyArray => Vec<PgMoney>,
    PgType::Void => (),
    PgType::Bool => bool,
    PgType::BoolArray => Vec<bool>,
    PgType::Float4 => f32,
    PgType::Float4Array => Vec<f32>,
    PgType::Float8 => f64,
    PgType::Int4Range => PgRange<i32>,
    PgType::Int8Range => PgRange<i64>,
    PgType::Text => String,
    PgType::TextArray => Vec<String>,
    PgType::Bpchar => String,
    PgType::BpcharArray => Vec<String>,
    PgType::Name => String,
    PgType::NameArray => Vec<String>,
    PgType::Varchar => String,
    PgType::VarcharArray => Vec<String>,
    PgType::Interval => PgInterval,
    PgType::IntervalArray => Vec<PgInterval>,
    PgType::Oid => Oid,
    PgType::OidArray => Vec<Oid>,
    PgType::Char => i8,
    PgType::CharArray => Vec<i8>,
    PgType::Int2 => i16,
    PgType::Int2Array => Vec<i16>,
    PgType::Int4 => i32,
    PgType::Int4Array => Vec<i32>,
    PgType::Int8 => i64,
    PgType::Int8Array => Vec<i64>,
    PgType::Timestamp => i64,
    PgType::TimestampArray => Vec<i64>,
    PgType::Time=> i64,
    PgType::TimeArray => Vec<i64>,
    PgType::Timestamptz => i64,
    PgType::TimestamptzArray => Vec<i64>,
    PgType::Date => i32,
    PgType::DateArray => Vec<i32>
}
