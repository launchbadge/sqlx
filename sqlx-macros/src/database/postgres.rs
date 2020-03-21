impl_database_ext! {
    sqlx::postgres::Postgres {
        bool,
        String | &str,
        i16,
        i32,
        i64,
        f32,
        f64,

        Vec<u8> | &[u8],

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

        #[cfg(feature = "time")]
        sqlx::types::time::Time,

        #[cfg(feature = "time")]
        sqlx::types::time::Date,

        #[cfg(feature = "time")]
        sqlx::types::time::PrimitiveDateTime,

        #[cfg(feature = "time")]
        sqlx::types::time::OffsetDateTime,

        #[cfg(feature = "bigdecimal")]
        sqlx::types::BigDecimal,

        #[cfg(feature = "ipnetwork")]
        sqlx::types::ipnetwork::IpNetwork,

        // Arrays
        Vec<bool> | &[bool],
        Vec<String> | &[String],
        Vec<i16> | &[i16],
        Vec<i32> | &[i32],
        Vec<i64> | &[i64],
        Vec<f32> | &[f32],
        Vec<f64> | &[f64],
    },
    ParamChecking::Strong,
    feature-types: info => info.type_feature_gate(),
    row = sqlx::postgres::PgRow
}
