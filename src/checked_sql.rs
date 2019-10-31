#[macro_export]
macro_rules! sql (
    ($sql:expr) => ({
        #[cfg(__sqlx_gather_queries)]
        const _: &'static str = $crate::checked_sql::__sqlx_checked_sql_noop($sql);
    })
);

pub const fn __sqlx_checked_sql_noop(sql: &'static str) -> &'static str {
    sql
}
