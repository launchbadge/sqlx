impl_database_ext! {
    sqlx::MySql {
        bool,
        String | &str,
        i16,
        i32,
        i64,
        f32,
        f64
    },
    ParamChecking::Weak
}
