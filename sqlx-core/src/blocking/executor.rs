use super::Runtime;
use crate::Database;

pub trait Executor<Rt: Runtime>: crate::Executor<Rt>
where
    Self::Database: Database<Rt>,
{
    fn execute<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> crate::Result<<Self::Database as Database<Rt>>::QueryResult>
    where
        'e: 'x,
        'q: 'x;

    fn fetch_all<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> crate::Result<Vec<<Self::Database as Database<Rt>>::Row>>
    where
        'e: 'x,
        'q: 'x;

    fn fetch_optional<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> crate::Result<Option<<Self::Database as Database<Rt>>::Row>>
    where
        'e: 'x,
        'q: 'x;
}
