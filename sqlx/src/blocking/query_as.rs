use crate::blocking::Executor;
use crate::{Blocking, Database, FromRow, QueryAs};

impl<'q, 'a, O, Db> QueryAs<'q, 'a, O, Db, Blocking>
where
    Db: Database,
    O: Send + Sync + FromRow<Db::Row>,
{
    pub fn fetch_optional<X>(&self, mut executor: X) -> crate::Result<Option<O>>
    where
        X: Executor<Blocking, Database = Db>,
    {
        Executor::fetch_optional(&mut executor, &self.inner)?.as_ref().map(O::from_row).transpose()
    }

    pub fn fetch_one<X>(&self, mut executor: X) -> crate::Result<O>
    where
        X: Executor<Blocking, Database = Db>,
    {
        O::from_row(&Executor::fetch_one(&mut executor, &self.inner)?)
    }

    pub fn fetch_all<X>(&self, mut executor: X) -> crate::Result<Vec<O>>
    where
        X: Executor<Blocking, Database = Db>,
    {
        Executor::fetch_all(&mut executor, &self.inner)?.iter().map(O::from_row).collect()
    }
}
