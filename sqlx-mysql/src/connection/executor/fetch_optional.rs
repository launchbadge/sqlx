use sqlx_core::Result;

use crate::connection::flush::QueryCommand;
use crate::protocol::{Query, QueryResponse, QueryStep, Status};
use crate::{MySqlConnection, MySqlRow};

macro_rules! impl_fetch_optional {
    ($(@$blocking:ident)? $self:ident, $sql:ident) => {{
        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        // send the server a text-based query that will be executed immediately
        // replies with ERR, OK, or a result set
        stream.write_packet(&Query { sql: $sql })?;

        // STATE: remember that we are now expecting a query response
        let cmd = QueryCommand::begin(commands);

        // default we did not find a row
        let mut first_row = None;

        #[allow(clippy::while_let_loop, unused_labels)]
        'results: loop {
            let ok = 'result: loop {
                match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                    QueryResponse::End(res) => break 'result res.into_result()?,
                    QueryResponse::ResultSet { columns } => {
                        let columns = recv_columns!($(@$blocking)? /* store = */ true, columns, stream, cmd);
                        log::debug!("columns = {:?}", columns);

                        'rows: loop {
                            match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                                // execute ignores any rows returned
                                // but we do increment affected rows
                                QueryStep::End(res) => break 'result res.into_result()?,
                                QueryStep::Row(row) => {
                                    first_row = Some(MySqlRow::new(row.deserialize_with(&columns[..])?));

                                    // get out as soon as possible after finding our one row
                                    break 'results;
                                }
                            }
                        }
                    }
                }
            };

            if !ok.status.contains(Status::MORE_RESULTS_EXISTS) {
                // STATE: the current command is complete
                commands.end();

                // no more results, time to finally call it quits and give up
                break;
            }

            // STATE: expecting a response from another statement
            *cmd = QueryCommand::QueryResponse;
        }

        Ok(first_row)
    }};
}

#[cfg(feature = "async")]
impl<Rt: sqlx_core::Async> MySqlConnection<Rt> {
    pub(super) async fn fetch_optional_async(&mut self, sql: &str) -> Result<Option<MySqlRow>> {
        flush!(self);
        impl_fetch_optional!(self, sql)
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> MySqlConnection<Rt> {
    pub(super) fn fetch_optional_blocking(&mut self, sql: &str) -> Result<Option<MySqlRow>> {
        flush!(@blocking self);
        impl_fetch_optional!(@blocking self, sql)
    }
}
