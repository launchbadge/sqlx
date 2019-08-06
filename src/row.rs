use crate::{
    postgres::protocol::DataRow,
    types::{FromSql, SqlType},
};

// TODO: Make this generic over backend
pub struct Row(pub(crate) DataRow);

impl Row {
    #[inline]
    pub fn get<ST, T>(&self, index: usize) -> T
    where
        ST: SqlType,
        T: FromSql<ST>,
    {
        T::from_sql(self.0.get(index).unwrap())
    }
}

pub trait FromRow<Record> {
    fn from_row(row: Row) -> Self;
}

impl<ST, T> FromRow<ST> for T
where
    ST: SqlType,
    T: FromSql<ST>,
{
    #[inline]
    fn from_row(row: Row) -> Self {
        row.get::<ST, T>(0)
    }
}

impl<ST1, T1> FromRow<(ST1,)> for (T1,)
where
    ST1: SqlType,
    T1: FromSql<ST1>,
{
    #[inline]
    fn from_row(row: Row) -> Self {
        (row.get::<ST1, T1>(0),)
    }
}

impl<ST1, ST2, T1, T2> FromRow<(ST1, ST2)> for (T1, T2)
where
    ST1: SqlType,
    ST2: SqlType,
    T1: FromSql<ST1>,
    T2: FromSql<ST2>,
{
    #[inline]
    fn from_row(row: Row) -> Self {
        (row.get::<ST1, T1>(0), row.get::<ST2, T2>(1))
    }
}
