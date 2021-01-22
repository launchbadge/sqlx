use sqlx_core::{Database, HasOutput, Runtime};

#[derive(Debug)]
pub struct Postgres;

impl<Rt> Database<Rt> for Postgres
where
    Rt: Runtime,
{
    type Connection = super::PostgresConnection<Rt>;
}

impl<'x> HasOutput<'x> for Postgres {
    type Output = &'x mut Vec<u8>;
}
