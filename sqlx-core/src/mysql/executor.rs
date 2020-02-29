use std::collections::HashMap;
use std::sync::Arc;

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;

use crate::describe::{Column, Describe};
use crate::executor::Executor;
use crate::mysql::protocol::{
    Capabilities, ColumnCount, ColumnDefinition, ComQuery, ComStmtExecute, ComStmtPrepare,
    ComStmtPrepareOk, Cursor, Decode, EofPacket, OkPacket, Row, TypeId,
};
use crate::mysql::{MySql, MySqlArguments, MySqlConnection, MySqlRow, MySqlTypeInfo};

enum Step {
    Command(u64),
    Row(Row),
}

enum OkOrResultSet {
    Ok(OkPacket),
    ResultSet(ColumnCount),
}

impl MySqlConnection {
    async fn ignore_columns(&mut self, count: usize) -> crate::Result<()> {
        for _ in 0..count {
            let _column = ColumnDefinition::decode(self.receive().await?.packet())?;
        }

        if count > 0 {
            self.receive_eof().await?;
        }

        Ok(())
    }

    async fn receive_ok_or_column_count(&mut self) -> crate::Result<OkOrResultSet> {
        self.receive().await?;

        match self.packet[0] {
            0x00 | 0xfe if self.packet.len() < 0xffffff => self.handle_ok().map(OkOrResultSet::Ok),
            0xff => self.handle_err(),

            _ => Ok(OkOrResultSet::ResultSet(ColumnCount::decode(
                self.packet(),
            )?)),
        }
    }

    async fn receive_column_types(&mut self, count: usize) -> crate::Result<Box<[TypeId]>> {
        let mut columns: Vec<TypeId> = Vec::with_capacity(count);

        for _ in 0..count {
            let column: ColumnDefinition =
                ColumnDefinition::decode(self.receive().await?.packet())?;

            columns.push(column.type_id);
        }

        if count > 0 {
            self.receive_eof().await?;
        }

        Ok(columns.into_boxed_slice())
    }

    async fn wait_for_ready(&mut self) -> crate::Result<()> {
        if self.next_seq_no != 0 {
            while let Some(_step) = self.step(&[], true).await? {
                // Drain steps until we hit the end
            }
        }

        Ok(())
    }

    async fn prepare(&mut self, query: &str) -> crate::Result<ComStmtPrepareOk> {
        // Start by sending a COM_STMT_PREPARE
        self.send(ComStmtPrepare { query }).await?;

        // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_com_stmt_prepare.html

        // First we should receive a COM_STMT_PREPARE_OK
        self.receive().await?;

        if self.packet[0] == 0xff {
            // Oops, there was an error in the prepare command
            return self.handle_err();
        }

        ComStmtPrepareOk::decode(self.packet())
    }

    async fn prepare_with_cache(&mut self, query: &str) -> crate::Result<u32> {
        if let Some(&id) = self.statement_cache.get(query) {
            Ok(id)
        } else {
            let prepare_ok = self.prepare(query).await?;

            // Remember our statement ID, so we do'd do this again the next time
            self.statement_cache
                .put(query.to_owned(), prepare_ok.statement_id);

            // Ignore input parameters
            self.ignore_columns(prepare_ok.params as usize).await?;

            // Collect output parameter names
            let mut columns = HashMap::with_capacity(prepare_ok.columns as usize);
            let mut index = 0_usize;
            for _ in 0..prepare_ok.columns {
                let column = ColumnDefinition::decode(self.receive().await?.packet())?;

                if let Some(name) = column.column_alias.or(column.column) {
                    columns.insert(name, index);
                }

                index += 1;
            }

            if prepare_ok.columns > 0 {
                self.receive_eof().await?;
            }

            // At the end of a command, this should go back to 0
            self.next_seq_no = 0;

            // Remember our column map in the statement cache
            self.statement_cache
                .put_columns(prepare_ok.statement_id, columns);

            Ok(prepare_ok.statement_id)
        }
    }

    // [COM_STMT_EXECUTE]
    async fn execute_statement(&mut self, id: u32, args: MySqlArguments) -> crate::Result<()> {
        self.send(ComStmtExecute {
            cursor: Cursor::NO_CURSOR,
            statement_id: id,
            params: &args.params,
            null_bitmap: &args.null_bitmap,
            param_types: &args.param_types,
        })
        .await
    }

    async fn step(&mut self, columns: &[TypeId], binary: bool) -> crate::Result<Option<Step>> {
        let capabilities = self.capabilities;
        ret_if_none!(self.try_receive().await?);

        match self.packet[0] {
            0xfe if self.packet.len() < 0xffffff => {
                // ResultSet row can begin with 0xfe byte (when using text protocol
                // with a field length > 0xffffff)

                if !capabilities.contains(Capabilities::DEPRECATE_EOF) {
                    let _eof = EofPacket::decode(self.packet())?;

                    // An EOF -here- signifies the end of the current command sequence
                    self.next_seq_no = 0;

                    Ok(None)
                } else {
                    self.handle_ok()
                        .map(|ok| Some(Step::Command(ok.affected_rows)))
                }
            }

            0xff => self.handle_err(),

            _ => Ok(Some(Step::Row(Row::decode(
                self.packet(),
                columns,
                binary,
            )?))),
        }
    }
}

impl MySqlConnection {
    pub(super) async fn execute_raw(&mut self, query: &str) -> crate::Result<()> {
        self.wait_for_ready().await?;

        self.send(ComQuery { query }).await?;

        // COM_QUERY can terminate before the result set with an ERR or OK packet
        let num_columns = match self.receive_ok_or_column_count().await? {
            OkOrResultSet::Ok(_) => {
                self.next_seq_no = 0;
                return Ok(());
            }

            OkOrResultSet::ResultSet(cc) => cc.columns as usize,
        };

        let columns = self.receive_column_types(num_columns as usize).await?;

        while let Some(_step) = self.step(&columns, false).await? {
            // Drop all responses
        }

        Ok(())
    }

    async fn execute(&mut self, query: &str, args: MySqlArguments) -> crate::Result<u64> {
        self.wait_for_ready().await?;

        let statement_id = self.prepare_with_cache(query).await?;

        self.execute_statement(statement_id, args).await?;

        // COM_STMT_EXECUTE can terminate before the result set with an ERR or OK packet
        let num_columns = match self.receive_ok_or_column_count().await? {
            OkOrResultSet::Ok(ok) => {
                self.next_seq_no = 0;

                return Ok(ok.affected_rows);
            }

            OkOrResultSet::ResultSet(cc) => cc.columns as usize,
        };

        self.ignore_columns(num_columns).await?;

        let mut res = 0;

        while let Some(step) = self.step(&[], true).await? {
            if let Step::Command(affected) = step {
                res = affected;
            }
        }

        Ok(res)
    }

    async fn describe(&mut self, query: &str) -> crate::Result<Describe<MySql>> {
        self.wait_for_ready().await?;

        let prepare_ok = self.prepare(query).await?;

        let mut param_types = Vec::with_capacity(prepare_ok.params as usize);
        let mut result_columns = Vec::with_capacity(prepare_ok.columns as usize);

        for _ in 0..prepare_ok.params {
            let param = ColumnDefinition::decode(self.receive().await?.packet())?;
            param_types.push(MySqlTypeInfo::from_column_def(&param));
        }

        if prepare_ok.params > 0 {
            self.receive_eof().await?;
        }

        for _ in 0..prepare_ok.columns {
            let column = ColumnDefinition::decode(self.receive().await?.packet())?;
            result_columns.push(Column::<MySql> {
                type_info: MySqlTypeInfo::from_column_def(&column),
                name: column.column_alias.or(column.column),
                table_id: column.table_alias.or(column.table),
            });
        }

        if prepare_ok.columns > 0 {
            self.receive_eof().await?;
        }

        // Command sequence is over
        self.next_seq_no = 0;

        Ok(Describe {
            param_types: param_types.into_boxed_slice(),
            result_columns: result_columns.into_boxed_slice(),
        })
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: MySqlArguments,
    ) -> BoxStream<'e, crate::Result<MySqlRow>> {
        Box::pin(async_stream::try_stream! {
            self.wait_for_ready().await?;

            let statement_id = self.prepare_with_cache(query).await?;

            let columns = self.statement_cache.get_columns(statement_id);

            self.execute_statement(statement_id, args).await?;

            // COM_STMT_EXECUTE can terminate before the result set with an ERR or OK packet
            let num_columns = match self.receive_ok_or_column_count().await? {
                OkOrResultSet::Ok(_) => {
                    self.next_seq_no = 0;
                    return;
                }

                OkOrResultSet::ResultSet(cc) => {
                    cc.columns as usize
                }
            };

            let column_types = self.receive_column_types(num_columns).await?;

            while let Some(Step::Row(row)) = self.step(&column_types, true).await? {
                yield MySqlRow { row, columns: Arc::clone(&columns) };
            }
        })
    }
}

impl Executor for MySqlConnection {
    type Database = super::MySql;

    fn send<'e, 'q: 'e>(&'e mut self, query: &'q str) -> BoxFuture<'e, crate::Result<()>> {
        Box::pin(self.execute_raw(query))
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: MySqlArguments,
    ) -> BoxFuture<'e, crate::Result<u64>> {
        Box::pin(self.execute(query, args))
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: MySqlArguments,
    ) -> BoxStream<'e, crate::Result<MySqlRow>> {
        self.fetch(query, args)
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>> {
        Box::pin(self.describe(query))
    }
}

impl_execute_for_query!(MySql);
