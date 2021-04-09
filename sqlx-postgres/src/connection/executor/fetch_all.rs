use std::sync::Arc;

use sqlx_core::{Execute, Result, Runtime};

use crate::protocol::backend::{BackendMessage, BackendMessageType, RowDescription};
use crate::{PgClientError, PgColumn, PgConnection, PgRow, Postgres};

impl<Rt: Runtime> PgConnection<Rt> {
    fn handle_message_in_fetch_all(
        &mut self,
        message: BackendMessage,
        rows: &mut Vec<PgRow>,
        columns: &mut Option<Arc<[PgColumn]>>,
    ) -> Result<bool> {
        match message.ty {
            BackendMessageType::BindComplete => {}

            BackendMessageType::DataRow => {
                rows.push(PgRow::new(message.deserialize()?, columns));
            }

            BackendMessageType::RowDescription => {
                *columns = Some(message.deserialize::<RowDescription>()?.columns.into());
            }

            // one statement has finished
            BackendMessageType::CommandComplete => {}

            BackendMessageType::ReadyForQuery => {
                self.handle_ready_for_query(message.deserialize()?);

                // all statements in this query have finished
                return Ok(true);
            }

            ty => {
                return Err(PgClientError::UnexpectedMessageType {
                    ty: ty as u8,
                    context: "executing a query [fetch_all]",
                }
                .into());
            }
        }

        Ok(false)
    }
}

macro_rules! impl_fetch_all {
    ($(@$blocking:ident)? $self:ident, $query:ident) => {{
        raw_query!($(@$blocking)? $self, $query);

        let mut rows = Vec::with_capacity(10);
        let mut columns = None;

        loop {
            let message = read_message!($(@$blocking)? $self.stream)?;

            if $self.handle_message_in_fetch_all(message, &mut rows, &mut columns)? {
                break;
            }
        }

        Ok(rows)
    }};
}

impl<Rt: Runtime> PgConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn fetch_all_async<'q, 'a, E>(&mut self, query: E) -> Result<Vec<PgRow>>
    where
        Rt: sqlx_core::Async,
        E: Execute<'q, 'a, Postgres>,
    {
        flush!(self);
        impl_fetch_all!(self, query)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn fetch_all_blocking<'q, 'a, E>(&mut self, query: E) -> Result<Vec<PgRow>>
    where
        Rt: sqlx_core::blocking::Runtime,
        E: Execute<'q, 'a, Postgres>,
    {
        flush!(@blocking self);
        impl_fetch_all!(@blocking self, query)
    }
}
