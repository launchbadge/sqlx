use sqlx_core::{Execute, Result, Runtime};

use crate::protocol::frontend::Query;
use crate::{PgConnection, PgRawValueFormat, Postgres};

macro_rules! impl_raw_query {
    ($(@$blocking:ident)? $self:ident, $query:ident) => {{
        let format = if let Some(arguments) = $query.arguments() {
            todo!("prepared query for postgres")
        } else {
            // directly execute the query as an unprepared, simple query
            $self.stream.write_message(&Query { sql: $query.sql() })?;

            // unprepared queries use the TEXT format
            // this is a significant waste of bandwidth for large result sets
            PgRawValueFormat::Text
        };

        // as we have written a SQL command of some kind to the stream
        // we now expect there to be an eventual ReadyForQuery
        // if for some reason the future for one of the execution methods is dropped
        // half-way through, we need to flush the stream until the ReadyForQuery point
        $self.pending_ready_for_query_count += 1;

        Ok(format)
    }};
}

impl<Rt: Runtime> PgConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn raw_query_async<'q, 'a, E>(&mut self, query: E) -> Result<PgRawValueFormat>
    where
        Rt: sqlx_core::Async,
        E: Execute<'q, 'a, Postgres>,
    {
        flush!(self);
        impl_raw_query!(self, query)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn raw_query_blocking<'q, 'a, E>(&mut self, query: E) -> Result<PgRawValueFormat>
    where
        Rt: sqlx_core::blocking::Runtime,
        E: Execute<'q, 'a, Postgres>,
    {
        flush!(@blocking self);
        impl_raw_query!(@blocking self, query)
    }
}

macro_rules! raw_query {
    (@blocking $self:ident, $sql:expr) => {
        $self.raw_query_blocking($sql)?
    };

    ($self:ident, $sql:expr) => {
        $self.raw_query_async($sql).await?
    };
}
