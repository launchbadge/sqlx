use crate::Database;

use super::Runtime;

pub trait Executor<Rt: Runtime>: crate::Executor<Rt>
where
    Self::Database: Database<Rt>,
{
    fn execute<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> crate::Result<()>
    where
        'e: 'x,
        'q: 'x;
}
