use super::fake_sqlx as sqlx;

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
        sqlx::types::chrono::NaiveDate,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveDateTime,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc> | sqlx::types::chrono::DateTime<_>,

        #[cfg(feature = "time")]
        sqlx::types::time::OffsetDateTime,

        #[cfg(feature = "time")]
        sqlx::types::time::PrimitiveDateTime,

        #[cfg(feature = "time")]
        sqlx::types::time::Date,

        #[cfg(feature = "uuid")]
        sqlx::types::Uuid,
    },
    ParamChecking::Weak,
    feature-types: _info => None,
    row: sqlx::sqlite::SqliteRow,
    // Since proc-macros don't benefit from async, we can make a describe call directly
    // which also ensures that the database is closed afterwards, regardless of errors.
    describe-blocking: sqlx_sqlite::describe_blocking,
}
