use sqlx_core::{Execute, Result, Runtime};

use crate::protocol::backend::{BackendMessage, BackendMessageType};
use crate::{PgClientError, PgConnection, PgQueryResult, Postgres};

impl<Rt: Runtime> PgConnection<Rt> {
    fn handle_message_in_execute(
        &mut self,
        message: BackendMessage,
        result: &mut PgQueryResult,
    ) -> Result<bool> {
        match message.ty {
            BackendMessageType::BindComplete => {}

            // ignore rows received or metadata about them
            // TODO: should we log a warning? its wasteful to use `execute` on a query
            //       that does return rows
            BackendMessageType::DataRow | BackendMessageType::RowDescription => {}

            BackendMessageType::CommandComplete => {
                // one statement has finished
                result.extend(Some(PgQueryResult::parse(message.contents)?));
            }

            BackendMessageType::ReadyForQuery => {
                self.handle_ready_for_query(message.deserialize()?);

                // all statements are finished
                return Ok(true);
            }

            ty => {
                return Err(PgClientError::UnexpectedMessageType {
                    ty: ty as u8,
                    context: "executing a query [execute]",
                }
                .into());
            }
        }

        Ok(false)
    }
}

macro_rules! impl_execute {
    ($(@$blocking:ident)? $self:ident, $query:ident) => {{
        raw_query!($(@$blocking)? $self, $query);

        let mut result = PgQueryResult::default();

        loop {
            let message = read_message!($(@$blocking)? $self.stream)?;

            if $self.handle_message_in_execute(message, &mut result)? {
                break;
            }
        }

        Ok(result)
    }};
}

impl<Rt: Runtime> PgConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn execute_async<'q, 'a, E>(&mut self, query: E) -> Result<PgQueryResult>
    where
        Rt: sqlx_core::Async,
        E: Execute<'q, 'a, Postgres>,
    {
        flush!(self);
        impl_execute!(self, query)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn execute_blocking<'q, 'a, E>(&mut self, query: E) -> Result<PgQueryResult>
    where
        Rt: sqlx_core::blocking::Runtime,
        E: Execute<'q, 'a, Postgres>,
    {
        flush!(@blocking self);
        impl_execute!(@blocking self, query)
    }
}
