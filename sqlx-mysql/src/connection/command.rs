use std::collections::VecDeque;

use sqlx_core::Runtime;

use super::MySqlConnection;

pub(crate) enum Command {
    // expecting [OkPacket]
    Simple,
    Query(QueryCommand),
}

pub(crate) struct QueryCommand {
    pub(crate) state: QueryState,
    pub(crate) columns: u64,
}

pub(crate) enum QueryState {
    // expecting [QueryResponse]
    QueryResponse,
    // expecting [QueryStep]
    QueryStep,
    // expecting [ColumnDefinition]
    ColumnDefinition { index: u64 },
}

pub(crate) fn begin_query_command(commands: &mut VecDeque<Command>) -> &mut QueryCommand {
    commands
        .push_back(Command::Query(QueryCommand { state: QueryState::QueryResponse, columns: 0 }));

    if let Some(Command::Query(query_cmd)) = commands.back_mut() {
        query_cmd
    } else {
        // UNREACHABLE: just pushed the query command
        unreachable!()
    }
}

impl<Rt: Runtime> MySqlConnection<Rt> {
    pub(crate) fn begin_simple_command(&mut self) {
        self.commands.push_back(Command::Simple);
    }

    pub(crate) fn end_command(&mut self) {
        self.commands.pop_front();
    }

    // pub(crate) fn flush_commands(&mut self) {
    //     // [...]
    // }
}
