use std::collections::VecDeque;
use std::hint::unreachable_unchecked;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

use sqlx_core::Result;

use crate::protocol::{PrepareResponse, QueryResponse, QueryStep, ResultPacket, Status};
use crate::{MySqlConnection, MySqlDatabaseError};

pub(crate) struct CommandQueue(pub(super) VecDeque<Command>);

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
}

#[derive(Debug)]
#[repr(u8)]
pub(crate) enum Command {
    Simple,
    Close,
    Query(QueryCommand),
    Prepare(PrepareCommand),
}

pub(crate) struct CommandGuard<'cmd, C> {
    queue: &'cmd mut CommandQueue,
    command: PhantomData<&'cmd mut C>,
    index: usize,
    ended: bool,
}

impl<'cmd, C> CommandGuard<'cmd, C> {
    fn begin(queue: &'cmd mut CommandQueue, command: Command) -> Self {
        let index = queue.0.len();
        queue.0.push_back(command);

        Self { queue, index, ended: false, command: PhantomData }
    }

    // called on successful command completion
    pub(crate) fn end(&mut self) {
        self.ended = true;
    }

    // on an error result, the command needs to end *normally* and pass
    // through the error to bubble
    pub(crate) fn end_if_error<T>(&mut self, res: Result<T>) -> Result<T> {
        match res {
            Ok(ok) => Ok(ok),
            Err(error) => {
                self.end();
                Err(error)
            }
        }
    }
}

impl<C> Drop for CommandGuard<'_, C> {
    fn drop(&mut self) {
        self.queue.end();

        if !self.ended {
            // if the command was not "completed" by success or a known
            // failure, we are in a **weird** state, queue up a close if
            // someone tries to re-use this connection
            self.queue.0.push_front(Command::Close);
        }
    }
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
    pub(crate) fn begin(queue: &mut CommandQueue) -> CommandGuard<'_, Self> {
        CommandGuard::begin(queue, Command::Query(Self::QueryResponse))
    }
}

impl Deref for CommandGuard<'_, QueryCommand> {
    type Target = QueryCommand;

    fn deref(&self) -> &Self::Target {
        if let Command::Query(cmd) = &self.queue.0[self.index] { cmd } else { unreachable!() }
    }
}

impl DerefMut for CommandGuard<'_, QueryCommand> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Command::Query(cmd) = &mut self.queue.0[self.index] { cmd } else { unreachable!() }
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
    pub(crate) fn begin(queue: &mut CommandQueue) -> CommandGuard<'_, Self> {
        CommandGuard::begin(queue, Command::Prepare(Self::PrepareResponse))
    }
}

impl Deref for CommandGuard<'_, PrepareCommand> {
    type Target = PrepareCommand;

    fn deref(&self) -> &Self::Target {
        if let Command::Prepare(cmd) = &self.queue.0[self.index] { cmd } else { unreachable!() }
    }
}

impl DerefMut for CommandGuard<'_, PrepareCommand> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Command::Prepare(cmd) = &mut self.queue.0[self.index] { cmd } else { unreachable!() }
    }
}
