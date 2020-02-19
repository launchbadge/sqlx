use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crate::executor::{Execute, Executor};
use crate::postgres::protocol::{self, Encode, Message, StatementId, TypeFormat};
use crate::postgres::{PgArguments, PgCursor, PgRow, PgTypeInfo, Postgres};

impl super::PgConnection {
    fn write_prepare(&mut self, query: &str, args: &PgArguments) -> StatementId {
        if let Some(&id) = self.statement_cache.get(query) {
            id
        } else {
            let id = StatementId(self.next_statement_id);
            self.next_statement_id += 1;

            protocol::Parse {
                statement: id,
                query,
                param_types: &*args.types,
            }
            .encode(self.stream.buffer_mut());

            self.statement_cache.put(query.to_owned(), id);

            id
        }
    }

    fn write_describe(&mut self, d: protocol::Describe) {
        d.encode(self.stream.buffer_mut())
    }

    fn write_bind(&mut self, portal: &str, statement: StatementId, args: &PgArguments) {
        protocol::Bind {
            portal,
            statement,
            formats: &[TypeFormat::Binary],
            // TODO: Early error if there is more than i16
            values_len: args.types.len() as i16,
            values: &*args.values,
            result_formats: &[TypeFormat::Binary],
        }
        .encode(self.stream.buffer_mut());
    }

    fn write_execute(&mut self, portal: &str, limit: i32) {
        protocol::Execute { portal, limit }.encode(self.stream.buffer_mut());
    }

    fn write_sync(&mut self) {
        protocol::Sync.encode(self.stream.buffer_mut());
    }
}

impl<'e> Executor<'e> for &'e mut super::PgConnection {
    type Database = Postgres;

    fn execute<'q, E>(self, query: E) -> PgCursor<'e>
    where
        E: Execute<'q, Self::Database>,
    {
        let (query, arguments) = query.into_parts();

        // TODO: Handle [arguments] being None. This should be a SIMPLE query.
        let arguments = arguments.unwrap();

        // Check the statement cache for a statement ID that matches the given query
        // If it doesn't exist, we generate a new statement ID and write out [Parse] to the
        // connection command buffer
        let statement = self.write_prepare(query, &arguments);

        // Next, [Bind] attaches the arguments to the statement and creates a named portal
        self.write_bind("", statement, &arguments);

        // Next, [Describe] will return the expected result columns and types
        // Conditionally run [Describe] only if the results have not been cached
        if !self.statement_cache.has_columns(statement) {
            self.write_describe(protocol::Describe::Portal(""));
        }

        // Next, [Execute] then executes the named portal
        self.write_execute("", 0);

        // Finally, [Sync] asks postgres to process the messages that we sent and respond with
        // a [ReadyForQuery] message when it's completely done. Theoretically, we could send
        // dozens of queries before a [Sync] and postgres can handle that. Execution on the server
        // is still serial but it would reduce round-trips. Some kind of builder pattern that is
        // termed batching might suit this.
        self.write_sync();

        PgCursor::from_connection(self, statement)
    }

    fn execute_by_ref<'q, E>(&mut self, query: E) -> PgCursor<'_>
    where
        E: Execute<'q, Self::Database>,
    {
        self.execute(query)
    }
}
