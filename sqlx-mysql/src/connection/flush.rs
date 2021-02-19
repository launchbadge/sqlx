use std::collections::VecDeque;
use std::hint::unreachable_unchecked;

use sqlx_core::Result;

use crate::protocol::{PrepareResponse, QueryResponse, QueryStep, ResultPacket, Status};
use crate::{MySqlConnection, MySqlDatabaseError};

pub(crate) struct CommandQueue(VecDeque<Command>);

impl CommandQueue {
    pub(crate) fn new() -> Self {
        Self(VecDeque::with_capacity(2))
    }

    // begin a simple command
    // in which we are expecting OK or ERR (a result)
    pub(crate) fn begin(&mut self) {
        self.0.push_back(Command::Simple);
    }
}

impl CommandQueue {
    pub(crate) fn end(&mut self) {
        self.0.pop_front();
    }

    fn maybe_end(&mut self, res: ResultPacket) {
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
        self.0.pop_front();
    }
}

#[derive(Debug)]
#[repr(u8)]
pub(crate) enum Command {
    // expecting [ResultPacket]
    Simple,
    Query(QueryCommand),
    Prepare(PrepareCommand),
}

#[derive(Debug)]
#[repr(u8)]
pub(crate) enum QueryCommand {
    // expecting [QueryResponse]
    QueryResponse,

    // expecting [QueryStep]
    QueryStep,

    // expecting {rem} more [ColumnDefinition] packets
    ColumnDefinition { rem: u16 },
}

impl QueryCommand {
    pub(crate) fn begin(queue: &mut CommandQueue) -> &mut Self {
        queue.0.push_back(Command::Query(Self::QueryResponse));

        if let Some(Command::Query(cmd)) = queue.0.back_mut() {
            cmd
        } else {
            // UNREACHABLE: just pushed a query command to the back of the vector, and we
            //              have &mut access, nobody else is pushing to it
            #[allow(unsafe_code)]
            unsafe {
                unreachable_unchecked()
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum PrepareCommand {
    // expecting [ERR] or [COM_STMT_PREPARE_OK]
    PrepareResponse,

    // expecting {rem} more [ColumnDefinition] packets for each parameter
    // stores {columns} as this state is before the [ColumnDefinition] state
    ParameterDefinition { rem: u16, columns: u16 },

    // expecting {rem} more [ColumnDefinition] packets for each parameter
    ColumnDefinition { rem: u16 },
}

impl PrepareCommand {
    pub(crate) fn begin(queue: &mut CommandQueue) -> &mut Self {
        queue.0.push_back(Command::Prepare(Self::PrepareResponse));

        if let Some(Command::Prepare(cmd)) = queue.0.back_mut() {
            cmd
        } else {
            // UNREACHABLE: just pushed a prepare command to the back of the vector, and we
            //              have &mut access, nobody else is pushing to it
            #[allow(unsafe_code)]
            unsafe {
                unreachable_unchecked()
            }
        }
    }
}

macro_rules! impl_flush {
    ($(@$blocking:ident)? $self:ident) => {{
        let Self { ref mut commands, ref mut stream, capabilities, .. } = *$self;

        log::debug!("flush!");

        while let Some(command) = commands.0.get_mut(0) {
            match command {
                Command::Simple => {
                    // simple commands where we expect an OK or ERR
                    // ex. COM_PING, COM_QUERY, COM_STMT_RESET, COM_SET_OPTION
                    commands.maybe_end(read_packet!($(@$blocking)? stream).deserialize_with(capabilities)?);
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

                                let _ = read_packet!($(@$blocking)? stream);

                                // STATE: now expecting the next parameter
                                *cmd = PrepareCommand::ParameterDefinition { rem: *rem - 1, columns: *columns };
                            }

                            PrepareCommand::ColumnDefinition { rem } => {
                                if *rem == 0 {
                                    // no more columns; done
                                    break commands.end();
                                }

                                let _ = read_packet!($(@$blocking)? stream);

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
                                    QueryResponse::End(end) => break commands.maybe_end(end),
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
                                    QueryStep::End(end) => break commands.maybe_end(end),
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
