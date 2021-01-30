use sqlx_core::Result;

use crate::connection::command::{begin_query_command, QueryState};
use crate::protocol::{Query, QueryResponse, QueryStep, Status};
use crate::{MySqlConnection, MySqlQueryResult};

macro_rules! impl_execute {
    ($(@$blocking:ident)? $self:ident, $sql:ident) => {{
        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        // send the server a text-based query that will be executed immediately
        // replies with ERR, OK, or a result set
        stream.write_packet(&Query { sql: $sql })?;

        // STATE: remember that we are now exepcting a query response
        let cmd = begin_query_command(commands);

        // default an empty query result
        // execute collects all discovered query results and SUMs
        // their values together
        let mut result = MySqlQueryResult::default();

        #[allow(clippy::while_let_loop, unused_labels)]
        'results: loop {
            let ok = 'result: loop {
                match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                    QueryResponse::Ok(ok) => break 'result ok,
                    QueryResponse::ResultSet { columns } => {
                        // acknowledge but discard any columns as execute returns no rows
                        recv_columns!($(@$blocking)? /* store = */ false, columns, stream, cmd);

                        'rows: loop {
                            match read_packet!($(@$blocking)? stream).deserialize_with((capabilities, &[][..]))? {
                                // execute ignores any rows returned
                                // but we do increment affected rows
                                QueryStep::Row(_row) => result.0.affected_rows += 1,
                                QueryStep::End(ok) => break 'result ok,
                            }
                        }
                    }
                }
            };

            // fold this into the total result for the SQL
            result.extend(Some(ok.into()));

            if !result.0.status.contains(Status::MORE_RESULTS_EXISTS) {
                // no more results, time to finally call it quits
                break;
            }

            // STATE: expecting a response from another statement
            cmd.state = QueryState::QueryResponse;
        }

        // STATE: the current command is complete
        $self.end_command();

        Ok(result)
    }};
}

#[cfg(feature = "async")]
impl<Rt: sqlx_core::Async> MySqlConnection<Rt> {
    pub(super) async fn execute_async(&mut self, sql: &str) -> Result<MySqlQueryResult> {
        impl_execute!(self, sql)
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> MySqlConnection<Rt> {
    pub(super) fn execute_blocking(&mut self, sql: &str) -> Result<MySqlQueryResult> {
        impl_execute!(@blocking self, sql)
    }
}
