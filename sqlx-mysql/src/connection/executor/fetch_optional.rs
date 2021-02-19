use sqlx_core::{Execute, Result, Runtime};

use crate::connection::command::QueryCommand;
use crate::protocol::{QueryResponse, QueryStep, Status};
use crate::{MySql, MySqlConnection, MySqlRow};

macro_rules! impl_fetch_optional {
    ($(@$blocking:ident)? $self:ident, $query:ident) => {{
        let format = raw_query!($(@$blocking)? $self, $query);

        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        // STATE: remember that we are now expecting a query response
        let mut cmd = QueryCommand::begin(commands);

        // default we did not find a row
        let mut first_row = None;

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
                                QueryStep::Row(row) => {
                                    first_row = Some(MySqlRow::new(row.deserialize_with((format, &columns[..]))?, &columns));

                                    // get out as soon as possible after finding our one row
                                    break 'results;
                                }
                            }
                        }
                    }
                }
            };

            // STATE: command is complete on error
            let ok = cmd.end_if_error(res)?;

            if !ok.status.contains(Status::MORE_RESULTS_EXISTS) {
                // STATE: the current command is complete
                cmd.end();

                // no more results, time to finally call it quits and give up
                break;
            }

            // STATE: expecting a response from another statement
            *cmd = QueryCommand::QueryResponse;
        }

        Ok(first_row)
    }};
}

impl<Rt: Runtime> MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn fetch_optional_async<'q, 'a, E>(
        &mut self,
        query: E,
    ) -> Result<Option<MySqlRow>>
    where
        Rt: sqlx_core::Async,
        E: Execute<'q, 'a, MySql>,
    {
        flush!(self);
        impl_fetch_optional!(self, query)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn fetch_optional_blocking<'q, 'a, E>(
        &mut self,
        query: E,
    ) -> Result<Option<MySqlRow>>
    where
        Rt: sqlx_core::blocking::Runtime,
        E: Execute<'q, 'a, MySql>,
    {
        flush!(@blocking self);
        impl_fetch_optional!(@blocking self, query)
    }
}
