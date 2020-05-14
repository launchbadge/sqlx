use sqlx_core as sqlx;

impl_database_ext! {
    sqlx::sqlite::Sqlite {
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
    row = sqlx::sqlite::SqliteRow,
    name = "SQLite"
}
