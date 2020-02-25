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

        // Arrays

        Vec<bool>, [bool],
        Vec<String>, [String],
        Vec<i16>, [i16],
        Vec<i32>, [i32],
        Vec<i64>, [i64],
        Vec<f32>, [f32],
        Vec<f64>, [f64],
    },
    ParamChecking::Strong
}
