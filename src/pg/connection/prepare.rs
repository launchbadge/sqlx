use super::RawConnection;
use crate::{
    postgres::{
        protocol::{self, BindValues},
        Postgres,
    },
    serialize::ToSql,
    types::{AsSql, SqlType},
};

pub struct Prepare<'a, 'b> {
    query: &'b str,
    pub(super) connection: &'a mut RawConnection,
    pub(super) bind: BindValues,
}

#[inline]
pub fn prepare<'a, 'b>(connection: &'a mut RawConnection, query: &'b str) -> Prepare<'a, 'b> {
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
    pub fn bind<T: AsSql<Postgres>>(mut self, value: T) -> Self
    where
        T: ToSql<Postgres, <T as AsSql<Postgres>>::Type>,
    {
        self.bind.add(value);
        self
    }

    #[inline]
    pub fn bind_as<ST: SqlType<Postgres>, T: ToSql<Postgres, ST>>(mut self, value: T) -> Self {
        self.bind.add_as::<ST, T>(value);
        self
    }

    pub(super) fn finish(self) -> &'a mut RawConnection {
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
