#[derive(Clone)]
pub enum TableType {
    Table(String),
}

impl Default for TableType {
    fn default() -> Self {
        TableType::Table(String::default())
    }
}

#[derive(Clone, Default)]
pub struct Table {
    pub(crate) table_type: TableType,
    pub(crate) alias: Option<String>,
    pub(crate) database: Option<String>,
}

impl From<String> for Table {
    fn from(s: String) -> Table {
        Table {
            table_type: TableType::Table(s),
            ..Table::default()
        }
    }
}

impl<'a> From<&'a str> for Table {
    fn from(s: &'a str) -> Table {
        Table {
            table_type: TableType::Table(s.into()),
            ..Table::default()
        }
    }
}

impl<'a> From<(&'a str, &'a str)> for Table {
    fn from(s: (&'a str, &'a str)) -> Table {
        Table {
            table_type: TableType::Table(s.0.into()),
            alias: Some(s.1.into()),
            ..Table::default()
        }
    }
}

impl<'a> From<(&'a str, &'a str, &'a str)> for Table {
    fn from(s: (&'a str, &'a str, &'a str)) -> Table {
        Table {
            table_type: TableType::Table(s.0.into()),
            alias: Some(s.1.into()),
            database: Some(s.2.into()),
        }
    }
}
