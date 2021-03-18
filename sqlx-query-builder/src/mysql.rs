use std::fmt::Write;

use sqlx_core::mysql::MySql;
use crate::{QueryBuilder, error, query, select::Select, table::{Table, TableType}};

impl QueryBuilder for MySql {
    const SYSTEM_IDENTIFIER_START: &'static str = "`";
    const SYSTEM_IDENTIFIER_END: &'static str = "`";

    fn build<T>(query: T) -> error::Result<(String, Vec<String>)>
    where
        T: Into<query::QueryType>,
    {
        match query.into() {
            query::QueryType::Select(select) => Self::build_select(select),
            _ => Ok(("*".into(), vec![])),
        }
    }

    fn build_select(select: Select) -> error::Result<(String, Vec<String>)> {
        let mut query = String::new();
        query.push_str("SELECT ");
        query.push_str(&select.columns.join(", "));
        query.push_str(" FROM ");
        query.push_str(
            &select
                .tables
                .into_iter()
                .map(|f| Self::build_table(f).unwrap())
                .collect::<Vec<String>>()
                .join(", "),
        );
        query.push_str(";");
        Ok((query, vec![]))
    }

    fn build_table(table: Table) -> error::Result<String> {
        let mut table_sql = String::new();

        if let Some(database) = table.database {
            table_sql.write_str(&database).unwrap();
            table_sql.write_str(".").unwrap();
        }

        match table.table_type {
            TableType::Table(s) => table_sql.write_str(&s).unwrap(),
        };

        if let Some(alias) = table.alias {
            table_sql.write_str(" as ").unwrap();
            table_sql.write_str(&alias).unwrap();
        }

        Ok(table_sql)
    }


}
