use super::{MariaDb, MariaDbQueryParameters, MariaDbRow};
use crate::{
    backend::Backend,
    describe::{Describe, ResultField},
    mariadb::protocol::{
        Capabilities, ColumnCountPacket, ColumnDefinitionPacket, ComStmtExecute, EofPacket,
        ErrPacket, OkPacket, ResultRow, StmtExecFlag,
    },
};
use futures_core::stream::BoxStream;

impl Backend for MariaDb {
    type QueryParameters = MariaDbQueryParameters;
    type Row = MariaDbRow;
    type TableIdent = String;

    async fn open(url: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        MariaDb::open(url).await
    }

    async fn close(mut self) -> crate::Result<()> {
        self.close().await
    }

    async fn ping(&mut self) -> crate::Result<()> {
        self.ping().await
    }

    async fn execute(&mut self, query: &str, params: MariaDbQueryParameters) -> crate::Result<u64> {
        // Write prepare statement to buffer
        self.start_sequence();
        let prepare_ok = self.send_prepare(query).await?;

        // SEND ================
        self.start_sequence();
        self.execute(prepare_ok.statement_id, params).await?;
        // =====================

        // Row Counter, used later
        let mut rows = 0u64;
        let capabilities = self.capabilities;
        let has_eof = capabilities.contains(Capabilities::CLIENT_DEPRECATE_EOF);

        let packet = self.receive().await?;
        if packet[0] == 0x00 {
            let _ok = OkPacket::decode(packet, capabilities)?;
        } else if packet[0] == 0xFF {
            return ErrPacket::decode(packet)?.expect_error();
        } else {
            // A Resultset starts with a [ColumnCountPacket] which is a single field that encodes
            // how many columns we can expect when fetching rows from this statement
            let column_count: u64 = ColumnCountPacket::decode(packet)?.columns;

            // Next we have a [ColumnDefinitionPacket] which verbosely explains each minute
            // detail about the column in question including table, aliasing, and type
            // TODO: This information was *already* returned by PREPARE .., is there a way to suppress generation
            let mut columns = vec![];
            for _ in 0..column_count {
                columns.push(ColumnDefinitionPacket::decode(self.receive().await?)?);
            }

            // When (legacy) EOFs are enabled, the fixed number column definitions are further terminated by
            // an EOF packet
            if !has_eof {
                let _eof = EofPacket::decode(self.receive().await?)?;
            }

            // For each row in the result set we will receive a ResultRow packet.
            // We may receive an [OkPacket], [EofPacket], or [ErrPacket] (depending on if EOFs are enabled) to finalize the iteration.
            loop {
                let packet = self.receive().await?;
                if packet[0] == 0xFE && packet.len() < 0xFF_FF_FF {
                    // NOTE: It's possible for a ResultRow to start with 0xFE (which would normally signify end-of-rows)
                    //       but it's not possible for an Ok/Eof to be larger than 0xFF_FF_FF.
                    if !has_eof {
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
        }

        Ok(rows)
    }

    fn fetch(
        &mut self,
        _query: &str,
        _params: MariaDbQueryParameters,
    ) -> BoxStream<'_, crate::Result<Self::Row>> {
        Box::pin(async_stream::try_stream! {
            // Write prepare statement to buffer
            self.start_sequence();
            let prepare_ok = self.send_prepare(query).await?;

            self.start_sequence();
            self.execute(prepare_ok.statement_id, params).await?;

            let capabilities = self.capabilities;
            let has_eof = capabilities.contains(Capabilities::CLIENT_DEPRECATE_EOF);

            let packet = self.receive().await?;
            if packet[0] == 0x00 {
                let _ok = OkPacket::decode(packet, capabilities)?;
            } else if packet[0] == 0xFF {
                return ErrPacket::decode(packet)?.expect_error();
            }
            // A Resultset starts with a [ColumnCountPacket] which is a single field that encodes
            // how many columns we can expect when fetching rows from this statement
            // let column_count: u64 = ColumnCountPacket::decode(packet)?.columns;

            // Next we have a [ColumnDefinitionPacket] which verbosely explains each minute
            // detail about the column in question including table, aliasing, and type
            // TODO: This information was *already* returned by PREPARE .., is there a way to suppress generation
            let mut columns = vec![];
            for _ in 0..column_count {
                columns.push(ColumnDefinitionPacket::decode(self.receive().await?)?);
            }

            // When (legacy) EOFs are enabled, the fixed number column definitions are further terminated by
            // an EOF packet
            // if !has_eof {
            //     let _eof = EofPacket::decode(self.receive().await?)?;
            // }

            // loop {
            //     let packet = self.receive().await?;
                // if packet[0] == 0xFE && packet.len() < 0xFF_FF_FF {
                //     if !has_eof {
                //         let _eof = EofPacket::decode(packet)?;
                //     } else {
                //         let _ok = OkPacket::decode(packet, capabilities)?;
                //     }
                //     break;
                // } else if packet[0] == 0xFF {
                //     let err = ErrPacket::decode(packet)?;
                //     panic!("received db err = {:?}", err);
                // } else {
                    // yield ResultRow::decode(packet, &columns);
                // }
            // }
        })
    }

    async fn fetch_optional(
        &mut self,
        _query: &str,
        _params: MariaDbQueryParameters,
    ) -> crate::Result<Option<Self::Row>> {
        unimplemented!();
    }

    async fn describe(&mut self, query: &str) -> crate::Result<Describe<MariaDb>> {
        let prepare_ok = self.send_prepare(query).await?;

        let mut param_types = Vec::with_capacity(prepare_ok.params as usize);

        for _ in 0..prepare_ok.params {
            let param = ColumnDefinitionPacket::decode(self.receive().await?)?;
            param_types.push(param.field_type.0);
        }

        self.check_eof().await?;

        let mut columns = Vec::with_capacity(prepare_ok.columns as usize);

        for _ in 0..prepare_ok.columns {
            let column = ColumnDefinitionPacket::decode(self.receive().await?)?;
            columns.push(ResultField {
                name: column.column_alias.or(column.column),
                table_id: column.table_alias.or(column.table),
                type_id: column.field_type.0,
            })
        }

        self.check_eof().await?;

        Ok(Describe {
            param_types,
            result_fields: columns,
        })
    }
}

impl_from_row_for_backend!(MariaDb);
impl_into_query_parameters_for_backend!(MariaDb);
