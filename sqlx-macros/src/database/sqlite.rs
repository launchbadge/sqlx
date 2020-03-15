impl_database_ext! {
    sqlx::sqlite::Sqlite {
        i32,
        i64,
        f32,
        f64,
        String,
        Vec<u8>,
    },
    ParamChecking::Weak,
    feature-types: info => None,
    row = sqlx::sqlite::SqliteRow
}
