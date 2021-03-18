use crate::{query::QueryType, table::Table};

#[derive(Clone, Default)]
pub struct Select {
    pub(crate) tables: Vec<Table>,
    pub(crate) columns: Vec<String>,
    pub(crate) conditions: Option<String>,
    pub(crate) ordering: Option<String>,
    pub(crate) grouping: Option<String>,
    pub(crate) having: Option<String>,
    pub(crate) limit: Option<String>,
    pub(crate) offset: Option<String>,
    pub(crate) joins: Option<String>,
    pub(crate) ctes: Option<String>,
    pub(crate) distinct: bool,
}

impl Select {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn select(mut self, colums: Vec<String>) -> Self {
        self.columns = colums;
        self
    }

    pub fn and_select(mut self, column: String) -> Self {
        self.columns.push(column);
        self
    }

    pub fn from<T>(mut self, table: T) -> Self
    where
        T: Into<Table>,
    {
        self.tables = vec![table.into()];
        self
    }

    pub fn and_from<T>(mut self, table: T) -> Self
    where
        T: Into<Table>,
    {
        self.tables.push(table.into());
        self
    }

    pub fn condition(mut self, condition: Option<String>) -> Self {
        self.conditions = condition;
        self
    }
}

impl Into<QueryType> for Select {
    fn into(self) -> QueryType {
        QueryType::Select(self)
    }
}
