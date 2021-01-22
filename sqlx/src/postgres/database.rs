use sqlx_core::HasOutput;

use super::PostgresConnection;
use crate::{Database, Runtime};

#[derive(Debug)]
pub struct Postgres;

impl<Rt: Runtime> Database<Rt> for Postgres {
    type Connection = PostgresConnection<Rt>;
}

impl<'x> HasOutput<'x> for Postgres {
    type Output = &'x mut Vec<u8>;
}
