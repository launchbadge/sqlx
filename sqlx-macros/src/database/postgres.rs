impl_database_ext! {
    sqlx::Postgres {
        bool,
        String,
        i16,
        i32,
        i64,
        f32,
        f64,

        // BYTEA
        Vec<u8>,

        #[cfg(feature = "uuid")]
        sqlx::types::Uuid,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx::types::chrono::NaiveTime,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx::types::chrono::NaiveDate,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx::types::chrono::NaiveDateTime,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc> | sqlx::types::chrono::DateTime<_>,

        #[cfg(feature = "time")]
        sqlx::types::time::Time,

        #[cfg(feature = "time")]
        sqlx::types::time::Date,

        #[cfg(feature = "time")]
        sqlx::types::time::PrimitiveDateTime,

        #[cfg(feature = "time")]
        sqlx::types::time::OffsetDateTime,
    },
    ParamChecking::Strong
}
