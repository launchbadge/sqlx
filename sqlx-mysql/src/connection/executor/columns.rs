use std::sync::Arc;

use sqlx_core::{Result, Runtime};

use crate::connection::command::QueryCommand;
use crate::protocol::{Capabilities, EofPacket};
use crate::stream::MySqlStream;
use crate::MySqlColumn;

macro_rules! impl_recv_columns {
    ($(@$blocking:ident)? $store:expr, $num_columns:ident, $stream:ident, $cmd:ident, $capabilities:ident) => {{
        #[allow(clippy::cast_possible_truncation)]
        let mut columns = if $store {
            Vec::<MySqlColumn>::with_capacity($num_columns as usize)
        } else {
            // we are going to drop column definitions, do not allocate
            Vec::new()
        };

        for (index, rem) in (1..=$num_columns).rev().enumerate() {
            // STATE: remember that we are expecting #rem more columns
            *$cmd = QueryCommand::ColumnDefinition { rem: rem.into() };

            // read in definition and only deserialize if we are saving
            // the column definitions

            let packet = read_packet!($(@$blocking)? $stream);

            if $store {
                columns.push(MySqlColumn::new(index, packet.deserialize()?));
            }
        }

        if $num_columns > 0 && !$capabilities.contains(Capabilities::DEPRECATE_EOF) {
            // in versions of MySQL before 5.7.5, an EOF packet is issued at the
            // end of the column list
            *$cmd = QueryCommand::ColumnDefinition { rem: 0 };
            let _eof: EofPacket = read_packet!($(@$blocking)? $stream).deserialize_with($capabilities)?;
        }

        // STATE: remember that we are now expecting a row or the end
        *$cmd = QueryCommand::QueryStep;

        Ok(columns.into())
    }};
}

impl<Rt: Runtime> MySqlStream<Rt> {
    #[cfg(feature = "async")]
    pub(super) async fn recv_columns_async(
        &mut self,
        store: bool,
        columns: u16,
        cmd: &mut QueryCommand,
        capabilities: Capabilities,
    ) -> Result<Arc<[MySqlColumn]>>
    where
        Rt: sqlx_core::Async,
    {
        impl_recv_columns!(store, columns, self, cmd, capabilities)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn recv_columns_blocking(
        &mut self,
        store: bool,
        columns: u16,
        cmd: &mut QueryCommand,
        capabilities: Capabilities,
    ) -> Result<Arc<[MySqlColumn]>>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_recv_columns!(@blocking store, columns, self, cmd, capabilities)
    }
}

macro_rules! recv_columns {
    (@blocking $store:expr, $columns:ident, $stream:ident, $cmd:ident, $capabilities:expr) => {
        $stream.recv_columns_blocking($store, $columns, &mut *$cmd, $capabilities)?
    };

    ($store:expr, $columns:ident, $stream:ident, $cmd:ident, $capabilities:expr) => {
        $stream.recv_columns_async($store, $columns, &mut *$cmd, $capabilities).await?
    };
}
