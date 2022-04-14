use sqlx_core as sqlx;

// f32 is not included below as REAL represents a floating point value
// stored as an 8-byte IEEE floating point number
// For more info see: https://www.sqlite.org/datatype3.html#storage_classes_and_datatypes
impl_database_ext! {
    sqlx::sqlite::Sqlite {
        bool,
        i32,
        i64,
        f64,
        String,
        Vec<u8>,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveDateTime,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc> | sqlx::types::chrono::DateTime<_>,
    },
    ParamChecking::Weak,
    feature-types: _info => None,
    row = sqlx::sqlite::SqliteRow,
    name = "SQLite"
}
