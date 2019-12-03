use super::MariaDb;
use crate::{
    backend::Backend,
    describe::{Describe, ResultField},
    executor::Executor,
    params::{IntoQueryParameters, QueryParameters},
    mariadb::protocol::{
        Capabilities, ColumnCountPacket, ColumnDefinitionPacket, ComStmtExecute, EofPacket,
        ErrPacket, OkPacket, ResultRow, StmtExecFlag,
    },
    mariadb::query::MariaDbQueryParameters,
    row::FromRow,
    url::Url,
};
use futures_core::{future::BoxFuture, stream::BoxStream};

impl Executor for MariaDb {
    type Backend = Self;

    fn execute<'e, 'q: 'e, I: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<u64>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
    {
        let params = params.into_params();

        Box::pin(async move {
            let prepare = self.send_prepare(query).await?;
            self.send_execute(prepare.statement_id, params).await?;

            let columns = self.column_definitions().await?;
            let capabilities = self.capabilities;

            // For each row in the result set we will receive a ResultRow packet.
            // We may receive an [OkPacket], [EofPacket], or [ErrPacket] (depending on if EOFs are enabled) to finalize the iteration.
            let mut rows = 0u64;
            loop {
                let packet = self.receive().await?;
                if packet[0] == 0xFE && packet.len() < 0xFF_FF_FF {
                    // NOTE: It's possible for a ResultRow to start with 0xFE (which would normally signify end-of-rows)
                    //       but it's not possible for an Ok/Eof to be larger than 0xFF_FF_FF.
                    if !capabilities.contains(Capabilities::CLIENT_DEPRECATE_EOF) {
                        let _eof = EofPacket::decode(packet)?;
                    } else {
                        let _ok = OkPacket::decode(packet, capabilities)?;
                    }

                    break;
                } else if packet[0] == 0xFF {
                    let err = ErrPacket::decode(packet)?;
                    panic!("received db err = {:?}", err);
                } else {
                    // Ignore result rows; exec only returns number of affected rows;
                    let _ = ResultRow::decode(packet, &columns)?;

                    // For every row we decode we increment counter
                    rows = rows + 1;
                }
            }

            Ok(rows)
        })
    }

    fn fetch<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxStream<'e, crate::Result<T>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send + Unpin,
    {
        let params = params.into_params();

         Box::pin(async_stream::try_stream! {
            let prepare = self.send_prepare(query).await?;
            self.send_execute(prepare.statement_id, params).await?;

            let columns = self.column_definitions().await?;
            let capabilities = self.capabilities;

            loop {
                let packet = self.receive().await?;
                if packet[0] == 0xFE && packet.len() < 0xFF_FF_FF {
                    // NOTE: It's possible for a ResultRow to start with 0xFE (which would normally signify end-of-rows)
                    //       but it's not possible for an Ok/Eof to be larger than 0xFF_FF_FF.
                    if !capabilities.contains(Capabilities::CLIENT_DEPRECATE_EOF) {
                        let _eof = EofPacket::decode(packet)?;
                    } else {
                        let _ok = OkPacket::decode(packet, capabilities)?;
                    }

                    break;
                } else if packet[0] == 0xFF {
                    let _err = ErrPacket::decode(packet)?;
                    panic!("ErrPacket received");
                } else {
                    let row = ResultRow::decode(packet, &columns)?;
                    yield FromRow::from_row(row);
                }
            }
         })
    }

    fn fetch_optional<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<Option<T>>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send,
    {
        let params = params.into_params();

        Box::pin(async move {
            let prepare = self.send_prepare(query).await?;
            self.send_execute(prepare.statement_id, params).await?;

            let columns = self.column_definitions().await?;
            let capabilities = self.capabilities;

            let mut row: Option<_> = None;

            loop {
                let packet = self.receive().await?;
                if packet[0] == 0xFE && packet.len() < 0xFF_FF_FF {
                    // NOTE: It's possible for a ResultRow to start with 0xFE (which would normally signify end-of-rows)
                    //       but it's not possible for an Ok/Eof to be larger than 0xFF_FF_FF.
                    if !capabilities.contains(Capabilities::CLIENT_DEPRECATE_EOF) {
                        let _eof = EofPacket::decode(packet)?;
                    } else {
                        let _ok = OkPacket::decode(packet, capabilities)?;
                    }

                    break;
                } else if packet[0] == 0xFF {
                    let _err = ErrPacket::decode(packet)?;
                } else {
                    row = Some(FromRow::from_row(ResultRow::decode(packet, &columns)?));
                }
            }

            Ok(row)
         })
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Backend>>> {
        unimplemented!();
    }
}
