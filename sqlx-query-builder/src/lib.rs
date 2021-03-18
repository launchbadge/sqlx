use select::Select;
use table::Table;

mod error;
mod mysql;
mod query;
mod select;
mod table;

pub trait QueryBuilder {
    const SYSTEM_IDENTIFIER_START: &'static str;
    const SYSTEM_IDENTIFIER_END: &'static str;

    fn build<T>(query: T) -> error::Result<(String, Vec<String>)>
    where
        T: Into<query::QueryType>;

    fn build_select(select: Select) -> error::Result<(String, Vec<String>)>;
    fn build_table(table: Table) -> error::Result<String>;
}

#[cfg(test)]
mod tests {
    use crate::{select::Select, QueryBuilder};
    use sqlx_core::mysql::MySql;

    #[test]
    fn simple_select_statement() {
        let (query, _) =
            MySql::build(Select::new().and_select("*".into()).and_from("sqlx")).unwrap();

        assert_eq!(query, "SELECT * FROM sqlx;".to_string());
    }
    #[test]
    fn select_with_table_alias() {
        let (query, _) = MySql::build(Select::new().and_select("*".into()).and_from(("sqlx", "s", "sqlx_db"))).unwrap();
        assert_eq!(query, "SELECT * FROM sqlx_db.sqlx as s;");
    }

    #[test]
    fn select_statement_with_column_names() {
        let (query, _) = MySql::build(Select::new().and_select("name".into()).from("sqlx")).unwrap();

        assert_eq!(query, "SELECT name FROM sqlx;".to_string());
    }
}
