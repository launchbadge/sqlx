use super::Runtime;
use crate::Database;

pub trait Executor<Rt: Runtime>: crate::Executor<Rt>
where
    Self::Database: Database,
{
    /// Execute the SQL query and return information about the result, including
    /// the number of rows affected, if any.
    fn execute<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> crate::Result<<Self::Database as Database>::QueryResult>
    where
        E: 'x + crate::Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x;

    fn fetch_all<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> crate::Result<Vec<<Self::Database as Database>::Row>>
    where
        E: 'x + crate::Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x;

    fn fetch_optional<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> crate::Result<Option<<Self::Database as Database>::Row>>
    where
        E: 'x + crate::Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x;

    fn fetch_one<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> crate::Result<<Self::Database as Database>::Row>
    where
        E: 'x + crate::Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        Executor::fetch_optional(self, query)?.ok_or(crate::Error::RowNotFound)
    }
}

impl<Rt: Runtime, X: Executor<Rt>> Executor<Rt> for &'_ mut X {
    fn execute<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> crate::Result<<Self::Database as Database>::QueryResult>
    where
        E: 'x + crate::Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        Executor::execute(&mut **self, query)
    }

    fn fetch_all<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> crate::Result<Vec<<Self::Database as Database>::Row>>
    where
        E: 'x + crate::Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        Executor::fetch_all(&mut **self, query)
    }

    fn fetch_optional<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> crate::Result<Option<<Self::Database as Database>::Row>>
    where
        E: 'x + crate::Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        Executor::fetch_optional(&mut **self, query)
    }
}
