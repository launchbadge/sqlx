use crate::{
    postgres::protocol::DataRow,
    types::{FromSql, SqlType},
};

// TODO: Make this generic over backend
pub struct Row(pub(crate) DataRow);

impl Row {
    pub fn get<ST, T>(&self, index: usize) -> T
    where
        ST: SqlType,
        T: FromSql<ST>,
    {
        T::from_sql(self.0.get(index).unwrap())
    }
}
