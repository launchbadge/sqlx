use sqlx_core::{Result, Runtime};

use crate::connection::command::{QueryCommand, QueryState};
use crate::protocol::ColumnDefinition;
use crate::stream::MySqlStream;

macro_rules! impl_recv_columns {
    ($(@$blocking:ident)? $num_columns:ident, $stream:ident, $cmd:ident) => {{
        #[allow(clippy::cast_possible_truncation)]
        let mut columns = Vec::<ColumnDefinition>::with_capacity($num_columns as usize);

        // STATE: remember how many columns are in this result set
        $cmd.columns = $num_columns;

        for index in 0..$num_columns {
            // STATE: remember that we are expecting the #index column definition
            $cmd.state = QueryState::ColumnDefinition { index };

            columns.push(read_packet!($(@$blocking)? $stream).deserialize()?);
        }

        // STATE: remember that we are now expecting a row or the end
        $cmd.state = QueryState::QueryStep;

        Ok(columns)
    }};
}

macro_rules! impl_recv_and_drop_columns {
    ($(@$blocking:ident)? $num_columns:ident, $stream:ident, $cmd:ident) => {{
        // STATE: remember how many columns are in this result set
        $cmd.columns = $num_columns;

        for index in 0..$num_columns {
            // STATE: remember that we are expecting the #index column definition
            $cmd.state = QueryState::ColumnDefinition { index };

            // read and immediately drop the column definition packet
            // this method is only invoked when we don't care about query results
            let _ = read_packet!($(@$blocking)? $stream);
        }

        // STATE: remember that we are now expecting a row or the end
        $cmd.state = QueryState::QueryStep;

        Ok(())
    }};
}

impl<Rt: Runtime> MySqlStream<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn recv_columns_async(
        &mut self,
        columns: u64,
        cmd: &mut QueryCommand,
    ) -> Result<Vec<ColumnDefinition>>
    where
        Rt: sqlx_core::Async,
    {
        impl_recv_columns!(columns, self, cmd)
    }

    #[cfg(feature = "async")]
    pub(super) async fn recv_and_drop_columns_async(
        &mut self,
        columns: u64,
        cmd: &mut QueryCommand,
    ) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        impl_recv_and_drop_columns!(columns, self, cmd)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn recv_columns_blocking(
        &mut self,
        columns: u64,
        cmd: &mut QueryCommand,
    ) -> Result<Vec<ColumnDefinition>>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_recv_columns!(@blocking columns, self, cmd)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn recv_and_drop_columns_blocking(
        &mut self,
        columns: u64,
        cmd: &mut QueryCommand,
    ) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_recv_and_drop_columns!(@blocking columns, self, cmd)
    }
}

macro_rules! recv_columns {
    (@blocking $columns:ident, $stream:ident, $cmd:ident) => {
        $stream.recv_columns_blocking($columns, $cmd)?
    };

    ($columns:ident, $stream:ident, $cmd:ident) => {
        $stream.recv_columns_async($columns, $cmd).await?
    };
}

macro_rules! recv_and_drop_columns {
    (@blocking $columns:ident, $stream:ident, $cmd:ident) => {
        $stream.recv_and_drop_columns_blocking($columns, $cmd)?
    };

    ($columns:ident, $stream:ident, $cmd:ident) => {
        $stream.recv_and_drop_columns_async($columns, $cmd).await?
    };
}
