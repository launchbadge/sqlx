use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crate::cursor::Cursor;
use crate::executor::{Execute, Executor};
use crate::postgres::protocol::{self, Encode, StatementId, TypeFormat};
use crate::postgres::{PgArguments, PgConnection, PgCursor, PgRow, PgTypeInfo, Postgres};

impl PgConnection {
    pub(crate) fn write_simple_query(&mut self, query: &str) {
        self.stream.write(protocol::Query(query));
    }

    pub(crate) fn write_prepare(&mut self, query: &str, args: &PgArguments) -> StatementId {
        // TODO: check query cache

        let id = StatementId(self.next_statement_id);

        self.next_statement_id += 1;

        self.stream.write(protocol::Parse {
            statement: id,
            query,
            param_types: &*args.types,
        });

        // TODO: write to query cache

        id
    }

    pub(crate) fn write_describe(&mut self, d: protocol::Describe) {
        self.stream.write(d);
    }

    pub(crate) fn write_bind(&mut self, portal: &str, statement: StatementId, args: &PgArguments) {
        self.stream.write(protocol::Bind {
            portal,
            statement,
            formats: &[TypeFormat::Binary],
            // TODO: Early error if there is more than i16
            values_len: args.types.len() as i16,
            values: &*args.values,
            result_formats: &[TypeFormat::Binary],
        });
    }

    pub(crate) fn write_execute(&mut self, portal: &str, limit: i32) {
        self.stream.write(protocol::Execute { portal, limit });
    }

    pub(crate) fn write_sync(&mut self) {
        self.stream.write(protocol::Sync);
    }
}

impl<'e> Executor<'e> for &'e mut super::PgConnection {
    type Database = Postgres;

    fn execute<'q, E>(self, query: E) -> PgCursor<'e, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        PgCursor::from_connection(self, query)
    }

    #[doc(hidden)]
    #[inline]
    fn execute_by_ref<'q, E>(&mut self, query: E) -> PgCursor<'_, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        self.execute(query)
    }
}

impl_execute_for_query!(Postgres);
