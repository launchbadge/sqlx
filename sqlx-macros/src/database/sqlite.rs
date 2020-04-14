impl_database_ext! {
    sqlx_core::sqlite::Sqlite {
        bool,
        i32,
        i64,
        f32,
        f64,
        String,
        Vec<u8>,
    },
    ParamChecking::Weak,
    feature-types: _info => None,
    row = sqlx_core::sqlite::SqliteRow,
    name = "SQLite"
}
