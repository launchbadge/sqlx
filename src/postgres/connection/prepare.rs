use super::Connection;
use crate::{
    postgres::protocol::{self, BindValues},
    types::{SqlType, ToSql, ToSqlAs},
};

pub struct Prepare<'a, 'b> {
    query: &'b str,
    pub(super) connection: &'a mut Connection,
    pub(super) bind: BindValues,
}

#[inline]
pub fn prepare<'a, 'b>(connection: &'a mut Connection, query: &'b str) -> Prepare<'a, 'b> {
    // TODO: Use a hash map to cache the parse
    // TODO: Use named statements
    Prepare {
        connection,
        query,
        bind: BindValues::new(),
    }
}

impl<'a, 'b> Prepare<'a, 'b> {
    #[inline]
    pub fn bind<T: ToSql>(mut self, value: T) -> Self
    where
        T: ToSqlAs<<T as ToSql>::Type>,
    {
        self.bind.add(value);
        self
    }

    #[inline]
    pub fn bind_as<ST: SqlType, T: ToSqlAs<ST>>(mut self, value: T) -> Self {
        self.bind.add_as::<ST, T>(value);
        self
    }

    pub(super) fn finish(self) -> &'a mut Connection {
        self.connection.write(protocol::Parse {
            portal: "",
            query: self.query,
            param_types: self.bind.types(),
        });

        self.connection.write(protocol::Bind {
            portal: "",
            statement: "",
            formats: self.bind.formats(),
            values_len: self.bind.values_len(),
            values: self.bind.values(),
            result_formats: &[1],
        });

        self.connection.write(protocol::Execute {
            portal: "",
            limit: 0,
        });

        self.connection.write(protocol::Sync);

        self.connection
    }
}
