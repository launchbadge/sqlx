use crate::arguments::Arguments;
use crate::encode::Encode;
use crate::mssql::database::MsSql;

#[derive(Default)]
pub struct MsSqlArguments {}

impl<'q> Arguments<'q> for MsSqlArguments {
    type Database = MsSql;

    fn reserve(&mut self, additional: usize, size: usize) {
        unimplemented!()
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'q + Encode<'q, Self::Database>,
    {
        unimplemented!()
    }
}
