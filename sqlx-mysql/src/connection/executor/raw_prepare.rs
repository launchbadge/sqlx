use sqlx_core::{Result, Runtime};

use crate::connection::command::PrepareCommand;
use crate::protocol::{ColumnDefinition, Prepare, PrepareResponse};
use crate::raw_statement::RawStatement;
use crate::{MySqlColumn, MySqlTypeInfo};

macro_rules! impl_raw_prepare {
    ($(@$blocking:ident)? $self:ident, $sql:ident) => {{
        let Self { ref mut stream, ref mut commands, capabilities, .. } = *$self;

        // send the server a query that to be prepared
        stream.write_packet(&Prepare { sql: $sql })?;

        // STATE: remember that we are now expecting a prepare response
        let mut cmd = PrepareCommand::begin(commands);

        let res = read_packet!($(@$blocking)? stream)
            .deserialize_with::<PrepareResponse, _>(capabilities)?.into_result();

        // STATE: command is complete on error
        let ok = cmd.end_if_error(res)?;

        let mut stmt = RawStatement::new(&ok);

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
        cmd.end();

        Ok(stmt)
    }};
}

impl<Rt: Runtime> super::MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn raw_prepare_async(&mut self, sql: &str) -> Result<RawStatement>
    where
        Rt: sqlx_core::Async,
    {
        flush!(self);
        impl_raw_prepare!(self, sql)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn raw_prepare_blocking(&mut self, sql: &str) -> Result<RawStatement>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        flush!(@blocking self);
        impl_raw_prepare!(@blocking self, sql)
    }
}

macro_rules! raw_prepare {
    (@blocking $self:ident, $sql:expr) => {
        $self.raw_prepare_blocking($sql)?
    };

    ($self:ident, $sql:expr) => {
        $self.raw_prepare_async($sql).await?
    };
}
