pub async fn generic_it_connects<'q, 'e, E>(e: E) -> sqlx::Result<i32>
where
    E: sqlx::Executor<'e>,
    E::Database: sqlx::Database,
    i32: sqlx::Type<Database = E::Database> + sqlx::Decode<'q, Database = E::Database>,
{
    sqlx::query("select 1 + 1")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(e)
        .await
}
