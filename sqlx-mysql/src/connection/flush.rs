use sqlx_core::{Error, Result};

use crate::connection::command::{Command, CommandQueue, PrepareCommand, QueryCommand};
use crate::protocol::{PrepareResponse, QueryResponse, QueryStep, ResultPacket, Status};
use crate::{MySqlConnection, MySqlDatabaseError};

fn maybe_end_with(queue: &mut CommandQueue, res: ResultPacket) {
    match res {
        ResultPacket::Ok(ok) => {
            if ok.status.contains(Status::MORE_RESULTS_EXISTS) {
                // an attached query response is next
                // we are still expecting one
                return;
            }
        }

        ResultPacket::Err(error) => {
            // without context, we should not bubble this err
            // log and continue forward
            log::error!("{}", MySqlDatabaseError(error));
        }
    }

    // STATE: end of query
    queue.0.pop_front();
}

macro_rules! impl_flush {
    ($(@$blocking:ident)? $self:ident) => {{
        let Self { ref mut commands, ref mut stream, ref mut closed, capabilities, .. } = *$self;

        while let Some(command) = commands.0.get_mut(0) {
            match command {
                Command::Close => {
                    if !*closed {
                        close!($(@$blocking)? stream);
                        *closed = true;
                    }

                    return Err(Error::Closed);
                }

                Command::Simple => {
                    // simple commands where we expect an OK or ERR
                    // ex. COM_PING, COM_QUERY, COM_STMT_RESET, COM_SET_OPTION
                    maybe_end_with(commands, read_packet!($(@$blocking)? stream).deserialize_with(capabilities)?);
                }

                Command::Prepare(ref mut cmd) => {
                    loop {
                        match cmd {
                            PrepareCommand::PrepareResponse => {
                                match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                                    PrepareResponse::Ok(ok) => {
                                        // STATE: expect the parameter definitions next
                                        *cmd = PrepareCommand::ParameterDefinition { rem: ok.params, columns: ok.columns };
                                    }

                                    PrepareResponse::Err(error) => {
                                        // without context, we should not bubble this err; log and continue forward
                                        log::error!("{}", MySqlDatabaseError(error));

                                        // STATE: end of command
                                        break commands.end();
                                    }
                                }
                            }

                            PrepareCommand::ParameterDefinition { rem, columns } => {
                                if *rem == 0 {
                                    // no more parameters
                                    // STATE: expect columns next
                                    *cmd = PrepareCommand::ColumnDefinition { rem: *columns };
                                    continue;
                                }

                                let _packet = read_packet!($(@$blocking)? stream);

                                // STATE: now expecting the next parameter
                                *cmd = PrepareCommand::ParameterDefinition { rem: *rem - 1, columns: *columns };
                            }

                            PrepareCommand::ColumnDefinition { rem } => {
                                if *rem == 0 {
                                    // no more columns; done
                                    break commands.end();
                                }

                                let _packet = read_packet!($(@$blocking)? stream);

                                // STATE: now expecting the next parameter
                                *cmd = PrepareCommand::ColumnDefinition { rem: *rem - 1 };
                            }
                        }
                    }
                }

                Command::Query(ref mut cmd) => {
                    loop {
                        match cmd {
                            // expecting OK, ERR, or a result set
                            QueryCommand::QueryResponse => {
                                match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                                    QueryResponse::End(end) => break maybe_end_with(commands, end),
                                    QueryResponse::ResultSet { columns } => {
                                        // STATE: expect the column definitions for each column
                                        *cmd = QueryCommand::ColumnDefinition { rem: columns };
                                    }
                                }
                            }

                            // expecting a column definition
                            // remembers how many more column definitions we need
                            QueryCommand::ColumnDefinition { rem } => {
                                if *rem == 0 {
                                    // no more parameters
                                    // STATE: now expecting OK (END), ERR, or a row
                                    *cmd = QueryCommand::QueryStep;
                                    continue;
                                }

                                let _ = read_packet!($(@$blocking)? stream);

                                // STATE: now expecting the next column
                                *cmd = QueryCommand::ColumnDefinition { rem: *rem - 1 };
                            }

                            // expecting OK, ERR, or a Row
                            QueryCommand::QueryStep => {
                                // either the query result set has ended or we receive
                                // and immediately drop a row
                                match read_packet!($(@$blocking)? stream).deserialize_with(capabilities)? {
                                    QueryStep::End(end) => break maybe_end_with(commands, end),
                                    QueryStep::Row(_) => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }};
}

#[cfg(feature = "async")]
impl<Rt: sqlx_core::Async> MySqlConnection<Rt> {
    pub(crate) async fn flush_async(&mut self) -> Result<()> {
        impl_flush!(self)
    }
}

#[cfg(feature = "blocking")]
impl<Rt: sqlx_core::blocking::Runtime> MySqlConnection<Rt> {
    pub(crate) fn flush_blocking(&mut self) -> Result<()> {
        impl_flush!(@blocking self)
    }
}

macro_rules! flush {
    (@blocking $self:ident) => {
        $self.flush_blocking()?
    };

    ($self:ident) => {
        $self.flush_async().await?
    };
}
