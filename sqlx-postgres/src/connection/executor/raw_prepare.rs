use sqlx_core::{placeholders, Result, Runtime};

use crate::protocol::backend::{
    BackendMessage, BackendMessageType, ParameterDescription, RowDescription,
};
use crate::protocol::frontend::{Describe, Parse, StatementId, Sync, Target};
use crate::raw_statement::RawStatement;
use crate::{PgArguments, PgClientError, PgConnection, Postgres};
use sqlx_core::arguments::ArgumentIndex;
use sqlx_core::placeholders::{ArgumentKind, Placeholder};

impl<Rt: Runtime> PgConnection<Rt> {
    fn start_raw_prepare(
        &mut self,
        sql: &placeholders::ParsedQuery<'_>,
        arguments: &PgArguments<'_>,
    ) -> Result<RawStatement> {
        let mut has_expansion = false;

        let sql =
            sql.expand::<Postgres, _>(placeholder_get_argument(arguments, &mut has_expansion))?;

        // if the query has a comma-expansion, we don't want to keep it as a named prepared statement
        let statement_id = if !has_expansion {
            let val = self.next_statement_id;
            self.next_statement_id = self.next_statement_id.wrapping_add(1);
            StatementId::Named(val)
        } else {
            StatementId::Unnamed
        };

        let statement = RawStatement::new(statement_id);

        self.stream.write_message(&Parse { statement: statement.id, sql: &sql, arguments })?;

        self.stream.write_message(&Describe { target: Target::Statement(statement.id) })?;

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
        sql: &placeholders::ParsedQuery<'_>,
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
        sql: &placeholders::ParsedQuery<'_>,
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

fn placeholder_get_argument<'b, 'a: 'b>(
    arguments: &'b PgArguments<'_>,
    has_expansion: &'b mut bool,
) -> impl FnMut(&ArgumentIndex<'_>, &Placeholder<'a>) -> Result<ArgumentKind, String> + 'b {
    move |idx, place| {
        // note: we don't need to print the argument cause it's included in the outer error
        let arg = arguments.get(idx).ok_or("unknown argument")?;

        Ok(if place.kleene.is_some() {
            let len = arg.value().vector_len().ok_or("expected vector for argument")?;

            *has_expansion = true;

            ArgumentKind::Vector(len)
        } else {
            ArgumentKind::Scalar
        })
    }
}
