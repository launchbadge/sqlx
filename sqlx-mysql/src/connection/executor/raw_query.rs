use sqlx_core::{Arguments, Execute, Result, Runtime};

use crate::protocol::{self, Query, QueryResponse, QueryStep, Status};
use crate::{MySql, MySqlConnection, MySqlRawValueFormat, MySqlRow};

macro_rules! impl_raw_query {
    ($(@$blocking:ident)? $self:ident, $query:ident) => {{
        let format = if let Some(arguments) = $query.arguments() {
            // prepare the statement for execution
            let statement = raw_prepare!($(@$blocking:ident)? $self, $query.sql());

            // execute the prepared statement
            $self.stream.write_packet(&protocol::Execute {
                statement: statement.id(),
                parameters: &statement.parameters,
                arguments: &arguments,
            })?;

            // prepared queries always use the BINARY format
            MySqlRawValueFormat::Binary
        } else {
            // directly execute the query as an unprepared, simple query
            $self.stream.write_packet(&Query { sql: $query.sql() })?;

            // unprepared queries use the TEXT format
            // this is a significant waste of bandwidth for large result sets
            MySqlRawValueFormat::Text
        };

        Ok(format)
    }};
}

impl<Rt: Runtime> MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn raw_query_async<'q, 'a, E>(
        &mut self,
        query: E,
    ) -> Result<MySqlRawValueFormat>
    where
        Rt: sqlx_core::Async,
        E: Execute<'q, 'a, MySql>,
    {
        flush!(self);
        impl_raw_query!(self, query)
    }

    #[cfg(feature = "blocking")]
    pub(super) fn raw_query_blocking<'q, 'a, E>(&mut self, query: E) -> Result<MySqlRawValueFormat>
    where
        Rt: sqlx_core::blocking::Runtime,
        E: Execute<'q, 'a, MySql>,
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
