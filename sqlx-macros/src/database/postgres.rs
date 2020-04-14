impl_database_ext! {
    sqlx_core::postgres::Postgres {
        bool,
        String | &str,
        i8,
        i16,
        i32,
        u32,
        i64,
        f32,
        f64,

        Vec<u8> | &[u8],

        #[cfg(feature = "uuid")]
        sqlx_core::types::Uuid,

        #[cfg(feature = "chrono")]
        sqlx_core::types::chrono::NaiveTime,

        #[cfg(feature = "chrono")]
        sqlx_core::types::chrono::NaiveDate,

        #[cfg(feature = "chrono")]
        sqlx_core::types::chrono::NaiveDateTime,

        #[cfg(feature = "chrono")]
        sqlx_core::types::chrono::DateTime<sqlx_core::types::chrono::Utc> | sqlx_core::types::chrono::DateTime<_>,

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

        #[cfg(feature = "ipnetwork")]
        sqlx_core::types::ipnetwork::IpNetwork,

        #[cfg(feature = "json")]
        serde_json::Value,

        // Arrays
        Vec<bool> | &[bool],
        Vec<String> | &[String],
        Vec<i8> | &[i8],
        Vec<i16> | &[i16],
        Vec<i32> | &[i32],
        Vec<u32> | &[u32],
        Vec<i64> | &[i64],
        Vec<f32> | &[f32],
        Vec<f64> | &[f64],


        #[cfg(feature = "uuid")]
        Vec<sqlx_core::types::Uuid> | &[sqlx_core::types::Uuid],

        #[cfg(feature = "chrono")]
        Vec<sqlx_core::types::chrono::NaiveTime> | &[sqlx_core::types::sqlx_core::types::chrono::NaiveTime],

        #[cfg(feature = "chrono")]
        Vec<sqlx_core::types::chrono::NaiveDate> | &[sqlx_core::types::chrono::NaiveDate],

        #[cfg(feature = "chrono")]
        Vec<sqlx_core::types::chrono::NaiveDateTime> | &[sqlx_core::types::chrono::NaiveDateTime],

        // TODO
        // #[cfg(feature = "chrono")]
        // Vec<sqlx_core::types::chrono::DateTime<sqlx_core::types::chrono::Utc>> | &[sqlx_core::types::chrono::DateTime<_>],

        #[cfg(feature = "time")]
        Vec<sqlx_core::types::time::Time> | &[sqlx_core::types::time::Time],

        #[cfg(feature = "time")]
        Vec<sqlx_core::types::time::Date> | &[sqlx_core::types::time::Date],

        #[cfg(feature = "time")]
        Vec<sqlx_core::types::time::PrimitiveDateTime> | &[sqlx_core::types::time::PrimitiveDateTime],

        #[cfg(feature = "time")]
        Vec<sqlx_core::types::time::OffsetDateTime> | &[sqlx_core::types::time::OffsetDateTime],

        #[cfg(feature = "bigdecimal")]
        Vec<sqlx_core::types::BigDecimal> | &[sqlx_core::types::BigDecimal],

        #[cfg(feature = "ipnetwork")]
        Vec<sqlx_core::types::ipnetwork::IpNetwork> | &[sqlx_core::types::ipnetwork::IpNetwork],

    },
    ParamChecking::Strong,
    feature-types: info => info.type_feature_gate(),
    row = sqlx_core::postgres::PgRow,
    name = "PostgreSQL"
}
