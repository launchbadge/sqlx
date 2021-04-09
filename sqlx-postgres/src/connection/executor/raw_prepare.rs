use sqlx_core::{Result, Runtime};

use crate::protocol::backend::{
    BackendMessage, BackendMessageType, ParameterDescription, RowDescription,
};
use crate::protocol::frontend::{Describe, Parse, StatementRef, Sync, Target};
use crate::raw_statement::RawStatement;
use crate::{PgArguments, PgClientError, PgConnection};

impl<Rt: Runtime> PgConnection<Rt> {
    fn start_raw_prepare(
        &mut self,
        sql: &str,
        arguments: &PgArguments<'_>,
    ) -> Result<RawStatement> {
        let statement_id = self.next_statement_id;
        self.next_statement_id = self.next_statement_id.wrapping_add(1);

        let statement = RawStatement::new(statement_id);

        self.stream.write_message(&Parse {
            statement: StatementRef::Named(statement.id),
            sql,
            arguments,
        })?;

        self.stream.write_message(&Describe {
            target: Target::Statement(StatementRef::Named(statement.id)),
        })?;

        self.stream.write_message(&Sync)?;

        self.pending_ready_for_query_count += 1;

        Ok(statement)
    }

    fn handle_message_in_raw_prepare(
        &mut self,
        message: BackendMessage,
        statement: &mut RawStatement,
    ) -> Result<bool> {
        match message.ty {
            // next message should be <ReadyForQuery>
            BackendMessageType::ParseComplete => {}

            BackendMessageType::ReadyForQuery => {
                self.handle_ready_for_query(message.deserialize()?);

                return Ok(true);
            }

            BackendMessageType::ParameterDescription => {
                let pd: ParameterDescription = message.deserialize()?;
                statement.parameters = pd.parameters;
            }

            BackendMessageType::RowDescription => {
                let rd: RowDescription = message.deserialize()?;
                statement.columns = rd.columns;
            }

            ty => {
                return Err(PgClientError::UnexpectedMessageType {
                    ty: ty as u8,
                    context: "preparing a query",
                }
                .into());
            }
        }

        Ok(false)
    }
}

macro_rules! impl_raw_prepare {
    ($(@$blocking:ident)? $self:ident, $sql:ident, $arguments:ident) => {{
        let mut statement = $self.start_raw_prepare($sql, $arguments)?;

        loop {
            let message = read_message!($(@$blocking)? $self.stream)?;

            if $self.handle_message_in_raw_prepare(message, &mut statement)? {
                break;
            }
        }

        Ok(statement)
    }};
}

impl<Rt: Runtime> super::PgConnection<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn raw_prepare_async(
        &mut self,
        sql: &str,
        arguments: &PgArguments<'_>,
    ) -> Result<RawStatement>
    where
        Rt: sqlx_core::Async,
    {
        flush!(self);
        impl_raw_prepare!(self, sql, arguments)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn raw_prepare_blocking(
        &mut self,
        sql: &str,
        arguments: &PgArguments<'_>,
    ) -> Result<RawStatement>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        flush!(@blocking self);
        impl_raw_prepare!(@blocking self, sql, arguments)
    }
}

macro_rules! raw_prepare {
    (@blocking $self:ident, $sql:expr, $arguments:expr) => {
        $self.raw_prepare_blocking($sql, $arguments)?
    };

    ($self:ident, $sql:expr, $arguments:expr) => {
        $self.raw_prepare_async($sql, $arguments).await?
    };
}
