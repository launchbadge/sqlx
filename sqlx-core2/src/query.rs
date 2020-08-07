use crate::database::Database;
use crate::error::Error;
use crate::arguments::Arguments;
use crate::execute::Execute;
use crate::executor::Executor;
use crate::to_value::ToValue;

pub struct Query<'q, DB: Database> {
    sql: &'q str,
    arguments: Arguments<'q, DB>,
}

impl<'q, DB: Database> Query<'q, DB> {
    pub fn bind<T: ToValue<DB>>(mut self, value: &'q T) -> Self {
        self.arguments.bind(value);
        self
    }

    pub fn bind_erased<T: ToValue<DB>>(mut self, value: &'q T) -> Self {
        self.arguments.bind(value);
        self
    }

    pub fn bind_unchecked<T: ToValue<DB>>(mut self, value: &'q T) -> Self {
        self.arguments.bind(value);
        self
    }

    #[inline]
    pub async fn execute<'e, E>(self, executor: E) -> Result<u64, Error>
    where
        E: Executor<'e, Database = DB>,
    {
        executor.execute(self).await
    }
}

impl<'q, DB: Database> Execute<'q, DB> for Query<'q, DB> {
    fn sql(&self) -> &'q str {
        self.sql
    }

    fn arguments(&'q mut self) -> Option<&'q Arguments<'q, DB>> {
        Some(&self.arguments)
    }
}

#[inline]
pub fn query<'q, DB: Database>(sql: &'q str) -> Query<'q, DB> {
    Query {
        sql,
        arguments: Arguments::new(),
    }
}
