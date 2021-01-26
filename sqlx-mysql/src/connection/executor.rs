#[cfg(feature = "async")]
use futures_util::{future::BoxFuture, FutureExt};
use sqlx_core::{Executor, Result, Runtime};

use super::command::{begin_query_command, QueryState};
use super::MySqlConnection;
use crate::protocol::{ColumnDefinition, Query, QueryResponse, QueryStep, Status};
use crate::MySql;

macro_rules! impl_execute {
    ($(@$blocking:ident)? $self:ident, $sql:ident) => {{
        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        stream.write_packet(&Query { sql: $sql })?;

        // STATE: remember that we are now exepcting a query response
        let cmd = begin_query_command(commands);

        #[allow(clippy::while_let_loop, unused_labels)]
        'results: loop {
            match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                QueryResponse::ResultSet { columns: num_columns } => {
                    #[allow(clippy::cast_possible_truncation)]
                    let mut columns = Vec::<ColumnDefinition>::with_capacity(num_columns as usize);

                    // STATE: remember how many columns are in this result set
                    cmd.columns = num_columns;

                    for index in 0..num_columns {
                        // STATE: remember that we are expecting the #index column definition
                        cmd.state = QueryState::ColumnDefinition { index };

                        columns.push(read_packet!($(@$blocking)? stream).deserialize()?);
                    }

                    // STATE: remember that we are now expecting a row or the end of the result set
                    cmd.state = QueryState::QueryStep;

                    'rows: loop {
                        match read_packet!($(@$blocking)? stream)
                            .deserialize_with((capabilities, columns.as_slice()))?
                        {
                            QueryStep::End(end) => {
                                // TODO: handle rowsaffected/matched - if any

                                if !end.status.contains(Status::MORE_RESULTS_EXISTS) {
                                    // TODO: STATE: the current command is complete

                                    break 'results;
                                }
                            }

                            QueryStep::Row(row) => {
                                // TODO: handle row
                            }
                        }
                    }
                }

                QueryResponse::Ok(ok) => {
                    // TODO: handle rows affected
                    // no rows possible to ever return
                    break;
                }
            }
        }

        Ok(())
    }};
}

#[cfg(feature = "async")]
impl<Rt: sqlx_core::Async> MySqlConnection<Rt> {
    async fn execute_async(&mut self, sql: &str) -> Result<()> {
        impl_execute!(self, sql)
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> MySqlConnection<Rt> {
    fn execute_blocking(&mut self, sql: &str) -> Result<()> {
        impl_execute!(@blocking self, sql)
    }
}

impl<Rt: Runtime> Executor<Rt> for MySqlConnection<Rt> {
    type Database = MySql;

    #[cfg(feature = "async")]
    fn execute<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> BoxFuture<'x, Result<()>>
    where
        Rt: sqlx_core::Async,
        'e: 'x,
        'q: 'x,
    {
        self.execute_async(sql).boxed()
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> sqlx_core::blocking::Executor<Rt> for MySqlConnection<Rt> {
    fn execute<'x, 'e, 'q>(&'e mut self, sql: &'q str) -> Result<()>
    where
        'e: 'x,
        'q: 'x,
    {
        self.execute_blocking(sql)
    }
}
