use sqlx::{ColumnIndex, Database, Decode, Executor, Row, Type};

pub async fn generic_it_connects<'conn, E>(e: E) -> sqlx::Result<i32>
where
    E: Executor<'conn>,
    E::Database: Database,
    for<'row> i32: Type<E::Database> + Decode<'row, E::Database>,
    <E::Database as Database>::Row: Row<Database = E::Database>,
    usize: ColumnIndex<<E::Database as Database>::Row>,
{
    sqlx::query("select 1 + 1")
        .try_map(|row: <E::Database as Database>::Row| row.try_get::<i32, _>(0))
        .fetch_one(e)
        .await
}
