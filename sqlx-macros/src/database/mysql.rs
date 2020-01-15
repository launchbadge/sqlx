impl_database_ext! {
    sqlx::MySql {
        String,
        // TODO: Add after the new type refactor
        // u8,
        // u16,
        // u32,
        // u64,
        i8,
        i16,
        i32,
        i64,
        f32,
        f64,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveTime,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveDate,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::NaiveDateTime,

        #[cfg(feature = "chrono")]
        sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,
    },
    ParamChecking::Weak
}
