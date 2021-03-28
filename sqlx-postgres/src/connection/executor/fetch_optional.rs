use std::sync::Arc;

use sqlx_core::{Execute, Result, Runtime};

use crate::protocol::backend::{BackendMessage, BackendMessageType, RowDescription};
use crate::{PgClientError, PgColumn, PgConnection, PgRow, Postgres};

impl<Rt: Runtime> PgConnection<Rt> {
    fn handle_message_in_fetch_optional(
        &mut self,
        message: BackendMessage,
        first_row: &mut Option<PgRow>,
        columns: &mut Option<Arc<[PgColumn]>>,
    ) -> Result<bool> {
        match message.ty {
            BackendMessageType::BindComplete => {}

            BackendMessageType::DataRow => {
                debug_assert!(first_row.is_none());

                *first_row = Some(PgRow::new(message.deserialize()?, &columns));

                // exit early, we have 1 row
                return Ok(true);
            }

            BackendMessageType::RowDescription => {
                *columns = Some(message.deserialize::<RowDescription>()?.columns.into());
            }

            BackendMessageType::CommandComplete => {
                // one statement has finished
            }

            BackendMessageType::ReadyForQuery => {
                self.handle_ready_for_query(message.deserialize()?);

                // all statements in this query have finished
                return Ok(true);
            }

            ty => {
                return Err(PgClientError::UnexpectedMessageType {
                    ty: ty as u8,
                    context: "executing a query [fetch_optional]",
                }
                .into());
            }
        }

        Ok(false)
    }
}

macro_rules! impl_fetch_optional {
    ($(@$blocking:ident)? $self:ident, $query:ident) => {{
        raw_query!($(@$blocking)? $self, $query);

        let mut first_row = None;
        let mut columns = None;

        loop {
            let message = read_message!($(@$blocking)? $self.stream)?;

            if $self.handle_message_in_fetch_optional(message, &mut first_row, &mut columns)? {
                break;
            }
        }

        Ok(first_row)
    }};
}

impl<Rt: Runtime> PgConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn fetch_optional_async<'q, 'a, E>(
        &mut self,
        query: E,
    ) -> Result<Option<PgRow>>
    where
        Rt: sqlx_core::Async,
        E: Execute<'q, 'a, Postgres>,
    {
        flush!(self);
        impl_fetch_optional!(self, query)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn fetch_optional_blocking<'q, 'a, E>(&mut self, query: E) -> Result<Option<PgRow>>
    where
        Rt: sqlx_core::blocking::Runtime,
        E: Execute<'q, 'a, Postgres>,
    {
        flush!(self);
        impl_fetch_optional!(@blocking self, query)
    }
}
