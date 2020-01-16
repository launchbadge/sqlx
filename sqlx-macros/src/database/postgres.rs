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

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveTime,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveDate,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveDateTime,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc> | sqlx::types::chrono::DateTime<_>,
    },
    ParamChecking::Strong
}
