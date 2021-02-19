use sqlx_core::{Execute, Result, Runtime};

use crate::connection::command::QueryCommand;
use crate::protocol::{Query, QueryResponse, QueryStep, Status};
use crate::{MySql, MySqlConnection, MySqlQueryResult};

macro_rules! impl_execute {
    ($(@$blocking:ident)? $self:ident, $query:ident) => {{
        raw_query!($(@$blocking)? $self, $query);

        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        // STATE: remember that we are now expecting a query response
        let mut cmd = QueryCommand::begin(commands);

        // default an empty query result
        // execute collects all discovered query results and SUMs
        // their values together
        let mut result = MySqlQueryResult::default();

        #[allow(clippy::while_let_loop, unused_labels)]
        'results: loop {
            let res = 'result: loop {
                match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                    QueryResponse::End(res) => break 'result res.into_result(),
                    QueryResponse::ResultSet { columns } => {
                        // acknowledge but discard any columns as execute returns no rows
                        recv_columns!($(@$blocking)? /* store = */ false, columns, stream, cmd);

                        'rows: loop {
                            match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                                // execute ignores any rows returned
                                // but we do increment affected rows
                                QueryStep::Row(_row) => result.0.affected_rows += 1,
                                QueryStep::End(res) => break 'result res.into_result(),
                            }
                        }
                    }
                }
            };

            // STATE: command is complete on error
            let ok = cmd.end_if_error(res)?;

            // fold this into the total result for the SQL
            result.extend(Some(ok.into()));

            if !result.0.status.contains(Status::MORE_RESULTS_EXISTS) {
                // no more results, time to finally call it quits
                break;
            }

            // STATE: expecting a response from another statement
            *cmd = QueryCommand::QueryResponse;
        }

        // STATE: the current command is complete
        cmd.end();

        Ok(result)
    }};
}

impl<Rt: Runtime> MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn execute_async<'q, 'a, E>(&mut self, query: E) -> Result<MySqlQueryResult>
    where
        Rt: sqlx_core::Async,
        E: Execute<'q, 'a, MySql>,
    {
        flush!(self);
        impl_execute!(self, query)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn execute_blocking<'q, 'a, E>(&mut self, query: E) -> Result<MySqlQueryResult>
    where
        Rt: sqlx_core::blocking::Runtime,
        E: Execute<'q, 'a, MySql>,
    {
        flush!(@blocking self);
        impl_execute!(@blocking self, query)
    }
}
