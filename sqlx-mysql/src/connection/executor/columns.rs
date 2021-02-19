use sqlx_core::{Result, Runtime};

use crate::connection::command::QueryCommand;
use crate::stream::MySqlStream;
use crate::MySqlColumn;

macro_rules! impl_recv_columns {
    ($(@$blocking:ident)? $store:expr, $num_columns:ident, $stream:ident, $cmd:ident) => {{
        #[allow(clippy::cast_possible_truncation)]
        let mut columns = if $store {
            Vec::<MySqlColumn>::with_capacity($num_columns as usize)
        } else {
            // we are going to drop column definitions, do not allocate
            Vec::new()
        };

        for (ordinal, rem) in (1..=$num_columns).rev().enumerate() {
            // STATE: remember that we are expecting #rem more columns
            *$cmd = QueryCommand::ColumnDefinition { rem };

            // read in definition and only deserialize if we are saving
            // the column definitions

            let packet = read_packet!($(@$blocking)? $stream);

            if $store {
                columns.push(MySqlColumn::new(ordinal, packet.deserialize()?));
            }
        }

        // STATE: remember that we are now expecting a row or the end
        *$cmd = QueryCommand::QueryStep;

        Ok(columns)
    }};
}

impl<Rt: Runtime> MySqlStream<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn recv_columns_async(
        &mut self,
        store: bool,
        columns: u16,
        cmd: &mut QueryCommand,
    ) -> Result<Vec<MySqlColumn>>
    where
        Rt: sqlx_core::Async,
    {
        impl_recv_columns!(store, columns, self, cmd)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn recv_columns_blocking(
        &mut self,
        store: bool,
        columns: u16,
        cmd: &mut QueryCommand,
    ) -> Result<Vec<MySqlColumn>>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_recv_columns!(@blocking store, columns, self, cmd)
    }
}

macro_rules! recv_columns {
    (@blocking $store:expr, $columns:ident, $stream:ident, $cmd:ident) => {
        $stream.recv_columns_blocking($store, $columns, &mut *$cmd)?
    };

    ($store:expr, $columns:ident, $stream:ident, $cmd:ident) => {
        $stream.recv_columns_async($store, $columns, &mut *$cmd).await?
    };
}
