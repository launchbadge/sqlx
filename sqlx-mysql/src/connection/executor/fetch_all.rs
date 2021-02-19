use sqlx_core::{Execute, Result, Runtime};

use crate::connection::command::QueryCommand;
use crate::protocol::{QueryResponse, QueryStep, Status};
use crate::{MySql, MySqlConnection, MySqlRow};

macro_rules! impl_fetch_all {
    ($(@$blocking:ident)? $self:ident, $query:ident) => {{
        let format = raw_query!($(@$blocking)? $self, $query);

        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        // STATE: remember that we are now expecting a query response
        let mut cmd = QueryCommand::begin(commands);

        // default an empty row set
        let mut rows = Vec::with_capacity(10);

        #[allow(clippy::while_let_loop, unused_labels)]
        'results: loop {
            let res = 'result: loop {
                match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                    QueryResponse::End(res) => break 'result res.into_result(),
                    QueryResponse::ResultSet { columns } => {
                        let columns = recv_columns!($(@$blocking)? /* store = */ true, columns, stream, cmd);

                        'rows: loop {
                            match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                                // execute ignores any rows returned
                                // but we do increment affected rows
                                QueryStep::End(res) => break 'result res.into_result(),
                                QueryStep::Row(row) => rows.push(MySqlRow::new(row.deserialize_with((format, &columns[..]))?, &columns)),
                            }
                        }
                    }
                }
            };

            // STATE: command is complete on error
            let ok = cmd.end_if_error(res)?;

            if !ok.status.contains(Status::MORE_RESULTS_EXISTS) {
                // no more results, time to finally call it quits
                break;
            }

            // STATE: expecting a response from another statement
            *cmd = QueryCommand::QueryResponse;
        }

        // STATE: the current command is complete
        cmd.end();

        Ok(rows)
    }};
}

impl<Rt: Runtime> MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn fetch_all_async<'q, 'a, E>(&mut self, query: E) -> Result<Vec<MySqlRow>>
    where
        Rt: sqlx_core::Async,
        E: Execute<'q, 'a, MySql>,
    {
        flush!(self);
        impl_fetch_all!(self, query)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn fetch_all_blocking<'q, 'a, E>(&mut self, query: E) -> Result<Vec<MySqlRow>>
    where
        Rt: sqlx_core::blocking::Runtime,
        E: Execute<'q, 'a, MySql>,
    {
        flush!(@blocking self);
        impl_fetch_all!(@blocking self, query)
    }
}
