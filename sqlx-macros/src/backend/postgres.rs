impl_backend_ext! {
    sqlx::Postgres {
        bool,
        String | &str,
        i16,
        i32,
        i64,
        f32,
        f64,
        #[cfg(feature = "uuid")]
        sqlx::types::Uuid
    }
}
