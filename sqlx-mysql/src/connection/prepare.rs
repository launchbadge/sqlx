use sqlx_core::{Result, Runtime};

use crate::connection::flush::PrepareCommand;
use crate::protocol::{ColumnDefinition, Prepare, PrepareResponse};
use crate::{MySqlColumn, MySqlStatement, MySqlTypeInfo};

macro_rules! impl_prepare {
    ($(@$blocking:ident)? $self:ident, $sql:ident) => {{
        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        // send the server a query that to be prepared
        stream.write_packet(&Prepare { sql: $sql })?;

        // STATE: remember that we are now expecting a prepare response
        let cmd = PrepareCommand::begin(commands);

        let res = read_packet!($(@$blocking)? stream)
            .deserialize_with::<PrepareResponse, _>(capabilities)?.into_result();

        let ok = match res {
            Ok(ok) => ok,
            Err(error) => {
                // STATE: prepare failed, command ended
                commands.end();

                return Err(error);
            },
        };

        let mut stmt = MySqlStatement::new(ok.statement_id);

        stmt.parameters.reserve(ok.params.into());
        stmt.columns.reserve(ok.columns.into());

        for index in (1..=ok.params).rev() {
            // STATE: remember that we are expecting #rem more columns
            *cmd = PrepareCommand::ParameterDefinition { rem: index, columns: ok.columns };

            let def = read_packet!($(@$blocking)? stream).deserialize()?;

            // extract the type only from the column definition
            // most other fields are useless
            stmt.parameters.push(MySqlTypeInfo::new(&def));
        }

        // TODO: handle EOF for old MySQL

        for (ordinal, rem) in (1..=ok.columns).rev().enumerate() {
            // STATE: remember that we are expecting #rem more columns
            *cmd = PrepareCommand::ColumnDefinition { rem };

            let def = read_packet!($(@$blocking)? stream).deserialize()?;

            stmt.columns.push(MySqlColumn::new(ordinal, def));
        }

        // TODO: handle EOF for old MySQL

        // STATE: the command is complete
        commands.end();

        Ok(stmt)
    }};
}

// TODO: should be private
impl<Rt: Runtime> super::MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub async fn prepare_async(&mut self, sql: &str) -> Result<MySqlStatement>
    where
        Rt: sqlx_core::Async,
    {
        flush!(self);
        impl_prepare!(self, sql)
    }

    #[cfg(feature = "blocking")]
    pub fn prepare_blocking(&mut self, sql: &str) -> Result<MySqlStatement>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        flush!(@blocking self);
        impl_prepare!(@blocking self, sql)
    }
}

macro_rules! prepare {
    (@blocking $self:ident, $sql:expr) => {
        $self.prepare_blocking($sql)?
    };

    ($self:ident, $sql:expr) => {
        $self.prepare_async($sql).await?
    };
}
