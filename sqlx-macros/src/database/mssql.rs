use sqlx_core as sqlx;

impl_database_ext! {
    sqlx::mssql::Mssql {
        bool,
        i8,
        i16,
        i32,
        i64,
        f32,
        f64,
        String,
        sqlx::types::chrono::NaiveDateTime,
        /*
        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx::types::chrono::NaiveTime,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx::types::chrono::NaiveDate,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx::types::chrono::NaiveDateTime,

        #[cfg(all(feature = "chrono", not(feature = "time")))]
        sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,
        */
    },
    ParamChecking::Weak,
    feature-types: _info => None,
    //feature-types: info => info.__type_feature_gate(),
    row = sqlx::mssql::MssqlRow,
    name = "MSSQL"
}
