use std::fmt::Display;

use crate::arguments::Arguments;
use crate::database::{Database, HasArguments};
use crate::encode::Encode;
use crate::query::Query;
use crate::types::Type;
use either::Either;
use std::marker::PhantomData;

pub struct QueryBuilder<'a, DB>
where
    DB: Database,
{
    query: String,
    arguments: Option<<DB as HasArguments<'a>>::Arguments>,
    variable_count: u16,
}

impl<'a, DB: Database> QueryBuilder<'a, DB>
where
    DB: Database,
{
    pub fn new(init: impl Into<String>) -> Self
    where
        <DB as HasArguments<'a>>::Arguments: Default,
    {
        QueryBuilder {
            query: init.into(),
            arguments: Some(Default::default()),
            variable_count: 0,
        }
    }

    pub fn push(&mut self, sql: impl Display) -> &mut Self {
        self.query.push_str(&format!(" {}", sql));

        self
    }

    pub fn push_bind<A>(&mut self, value: A) -> &mut Self
    where
        A: 'a + Encode<'a, DB> + Send + Type<DB>,
    {
        match self.arguments {
            Some(ref mut arguments) => {
                arguments.add(value);
                self.variable_count += 1;
                self.query
                    .push_str(&arguments.place_holder(self.variable_count));
            }
            None => panic!("Arguments taken already"),
        }

        self
    }

    pub fn build(&mut self) -> Query<'_, DB, <DB as HasArguments<'a>>::Arguments> {
        Query {
            statement: Either::Left(&self.query),
            arguments: match self.arguments.take() {
                Some(arguments) => Some(arguments),
                None => None,
            },
            database: PhantomData,
            persistent: true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::postgres::Postgres;

    #[test]
    fn test_new() {
        let qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM users");
        assert_eq!(qb.query, "SELECT * FROM users");
    }

    #[test]
    fn test_push() {
        let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM users");
        let second_line = "WHERE last_name LIKE '[A-N]%;";
        qb.push(second_line);

        assert_eq!(
            qb.query,
            "SELECT * FROM users WHERE last_name LIKE '[A-N]%;".to_string(),
        );
    }

    #[test]
    fn test_push_bind() {
        let mut qb: QueryBuilder<'_, Postgres> =
            QueryBuilder::new("SELECT * FROM users WHERE id = ");

        qb.push_bind(42i32)
            .push("OR membership_level = ")
            .push_bind(3i32);

        assert_eq!(
            qb.query,
            "SELECT * FROM users WHERE id = $1 OR membership_level = $2"
        );
        assert_eq!(qb.variable_count, 2);
    }

    #[test]
    fn test_push_bind_handles_strings() {
        let mut qb: QueryBuilder<'_, Postgres> =
            QueryBuilder::new("SELECT * FROM users WHERE id = ");

        qb.push_bind(42i32)
            .push("OR last_name = ")
            .push_bind("'Doe'")
            .push("AND membership_level = ")
            .push_bind(3i32);

        assert_eq!(
            qb.query,
            "SELECT * FROM users WHERE id = $1 OR last_name = $2 AND membership_level = $3"
        );
        assert_eq!(qb.variable_count, 3);
    }

    #[test]
    fn test_build() {
        let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("SELECT * FROM users");

        qb.push("WHERE id = ").push_bind(42i32);
        let query = qb.build();

        assert_eq!(
            query.statement.unwrap_left(),
            "SELECT * FROM users WHERE id = $1"
        );
        assert_eq!(query.persistent, true);
    }
}
