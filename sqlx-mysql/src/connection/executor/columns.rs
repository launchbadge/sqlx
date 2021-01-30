use sqlx_core::{Result, Runtime};

use crate::connection::command::{QueryCommand, QueryState};
use crate::protocol::ColumnDefinition;
use crate::stream::MySqlStream;

macro_rules! impl_recv_columns {
    ($(@$blocking:ident)? $store:expr, $num_columns:ident, $stream:ident, $cmd:ident) => {{
        #[allow(clippy::cast_possible_truncation)]
        let mut columns = if $store {
            Vec::<ColumnDefinition>::with_capacity($num_columns as usize)
        } else {
            // we are going to drop column definitions, do not allocate
            Vec::new()
        };

        // STATE: remember how many columns are in this result set
        $cmd.columns = $num_columns;

        for index in 0..$num_columns {
            // STATE: remember that we are expecting the #index column definition
            $cmd.state = QueryState::ColumnDefinition { index };

            // read in definition and only deserialize if we are saving
            // the column definitions

            let packet = read_packet!($(@$blocking)? $stream);

            if $store {
                columns.push(packet.deserialize()?);
            }
        }

        // STATE: remember that we are now expecting a row or the end
        $cmd.state = QueryState::QueryStep;

        Ok(columns)
    }};
}

impl<Rt: Runtime> MySqlStream<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn recv_columns_async(
        &mut self,
        store: bool,
        columns: u64,
        cmd: &mut QueryCommand,
    ) -> Result<Vec<ColumnDefinition>>
    where
        Rt: sqlx_core::Async,
    {
        impl_recv_columns!(store, columns, self, cmd)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn recv_columns_blocking(
        &mut self,
        store: bool,
        columns: u64,
        cmd: &mut QueryCommand,
    ) -> Result<Vec<ColumnDefinition>>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_recv_columns!(@blocking store, columns, self, cmd)
    }
}

macro_rules! recv_columns {
    (@blocking $store:expr, $columns:ident, $stream:ident, $cmd:ident) => {
        $stream.recv_columns_blocking($store, $columns, $cmd)?
    };

    ($store:expr, $columns:ident, $stream:ident, $cmd:ident) => {
        $stream.recv_columns_async($store, $columns, $cmd).await?
    };
}
