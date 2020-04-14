impl_database_ext! {
    sqlx_core::mysql::MySql {
        u8,
        u16,
        u32,
        u64,
        i8,
        i16,
        i32,
        i64,
        f32,
        f64,

        // CHAR, VAR_CHAR, TEXT
        String,

        // BINARY, VAR_BINARY, BLOB
        Vec<u8>,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx_core::types::chrono::NaiveTime,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx_core::types::chrono::NaiveDate,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx_core::types::chrono::NaiveDateTime,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx_core::types::chrono::DateTime<sqlx_core::types::chrono::Utc>,

        #[cfg(feature = "time")]
        sqlx_core::types::time::Time,

        #[cfg(feature = "time")]
        sqlx_core::types::time::Date,

        #[cfg(feature = "time")]
        sqlx_core::types::time::PrimitiveDateTime,

        #[cfg(feature = "time")]
        sqlx_core::types::time::OffsetDateTime,

        #[cfg(feature = "bigdecimal")]
        sqlx_core::types::BigDecimal,
    },
    ParamChecking::Weak,
    feature-types: info => info.type_feature_gate(),
    row = sqlx_core::mysql::MySqlRow,
    name = "MySQL/MariaDB"
}
