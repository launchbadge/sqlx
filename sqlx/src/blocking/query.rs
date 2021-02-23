use crate::blocking::Executor;
use crate::{Blocking, Database, Query};

impl<'q, 'a, Db: Database> Query<'q, 'a, Db, Blocking> {
    pub fn execute<X>(&self, mut executor: X) -> crate::Result<Db::QueryResult>
    where
        X: Executor<Blocking, Database = Db>,
    {
        Executor::execute(&mut executor, self)
    }

    pub fn fetch_optional<X>(&self, mut executor: X) -> crate::Result<Option<Db::Row>>
    where
        X: Executor<Blocking, Database = Db>,
    {
        Executor::fetch_optional(&mut executor, self)
    }

    pub fn fetch_one<X>(&self, mut executor: X) -> crate::Result<Db::Row>
    where
        X: Executor<Blocking, Database = Db>,
    {
        Executor::fetch_one(&mut executor, self)
    }

    pub fn fetch_all<X>(&self, mut executor: X) -> crate::Result<Vec<Db::Row>>
    where
        X: Executor<Blocking, Database = Db>,
    {
        Executor::fetch_all(&mut executor, self)
    }
}
