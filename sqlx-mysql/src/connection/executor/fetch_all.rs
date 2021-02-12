use sqlx_core::Result;

use crate::connection::flush::QueryCommand;
use crate::protocol::{Query, QueryResponse, QueryStep, Status};
use crate::{MySqlConnection, MySqlRow};

macro_rules! impl_fetch_all {
    ($(@$blocking:ident)? $self:ident, $sql:ident) => {{
        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        // send the server a text-based query that will be executed immediately
        // replies with ERR, OK, or a result set
        stream.write_packet(&Query { sql: $sql })?;

        // STATE: remember that we are now expecting a query response
        let cmd = QueryCommand::begin(commands);

        // default an empty row set
        let mut rows = Vec::with_capacity(10);

        #[allow(clippy::while_let_loop, unused_labels)]
        'results: loop {
            let ok = 'result: loop {
                match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                    QueryResponse::End(res) => break 'result res.into_result()?,
                    QueryResponse::ResultSet { columns } => {
                        let columns = recv_columns!($(@$blocking)? /* store = */ true, columns, stream, cmd);

                        'rows: loop {
                            match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                                // execute ignores any rows returned
                                // but we do increment affected rows
                                QueryStep::End(res) => break 'result res.into_result()?,
                                QueryStep::Row(row) => rows.push(MySqlRow::new(row.deserialize_with(&columns[..])?)),
                            }
                        }
                    }
                }
            };

            if !ok.status.contains(Status::MORE_RESULTS_EXISTS) {
                // no more results, time to finally call it quits
                break;
            }

            // STATE: expecting a response from another statement
            *cmd = QueryCommand::QueryResponse;
        }

        // STATE: the current command is complete
        commands.end();

        Ok(rows)
    }};
}

#[cfg(feature = "async")]
impl<Rt: sqlx_core::Async> MySqlConnection<Rt> {
    pub(super) async fn fetch_all_async(&mut self, sql: &str) -> Result<Vec<MySqlRow>> {
        flush!(self);
        impl_fetch_all!(self, sql)
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> MySqlConnection<Rt> {
    pub(super) fn fetch_all_blocking(&mut self, sql: &str) -> Result<Vec<MySqlRow>> {
        flush!(@blocking self);
        impl_fetch_all!(@blocking self, sql)
    }
}
