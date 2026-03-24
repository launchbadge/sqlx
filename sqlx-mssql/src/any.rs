use crate::{
    Mssql, MssqlColumn, MssqlConnectOptions, MssqlConnection, MssqlQueryResult, MssqlRow,
    MssqlTransactionManager, MssqlTypeInfo,
};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{stream, FutureExt, StreamExt, TryStreamExt};
use sqlx_core::any::{
    AnyArguments, AnyColumn, AnyConnectOptions, AnyConnectionBackend, AnyQueryResult, AnyRow,
    AnyStatement, AnyTypeInfo, AnyTypeInfoKind,
};
use sqlx_core::connection::Connection;
use sqlx_core::database::Database;
use sqlx_core::executor::Executor;
use sqlx_core::sql_str::SqlStr;
use sqlx_core::transaction::TransactionManager;
use std::future;

sqlx_core::declare_driver_with_optional_migrate!(DRIVER = Mssql);

impl AnyConnectionBackend for MssqlConnection {
    fn name(&self) -> &str {
        <Mssql as Database>::NAME
    }

    fn close(self: Box<Self>) -> BoxFuture<'static, sqlx_core::Result<()>> {
        Connection::close(*self).boxed()
    }

    fn close_hard(self: Box<Self>) -> BoxFuture<'static, sqlx_core::Result<()>> {
        Connection::close_hard(*self).boxed()
    }

    fn ping(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        Connection::ping(self).boxed()
    }

    fn begin(&mut self, statement: Option<SqlStr>) -> BoxFuture<'_, sqlx_core::Result<()>> {
        MssqlTransactionManager::begin(self, statement).boxed()
    }

    fn commit(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        MssqlTransactionManager::commit(self).boxed()
    }

    fn rollback(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        MssqlTransactionManager::rollback(self).boxed()
    }

    fn start_rollback(&mut self) {
        MssqlTransactionManager::start_rollback(self)
    }

    fn get_transaction_depth(&self) -> usize {
        MssqlTransactionManager::get_transaction_depth(self)
    }

    fn shrink_buffers(&mut self) {
        Connection::shrink_buffers(self);
    }

    fn flush(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        Connection::flush(self).boxed()
    }

    fn should_flush(&self) -> bool {
        Connection::should_flush(self)
    }

    #[cfg(feature = "migrate")]
    fn as_migrate(
        &mut self,
    ) -> sqlx_core::Result<&mut (dyn sqlx_core::migrate::Migrate + Send + 'static)> {
        Ok(self)
    }

    fn fetch_many(
        &mut self,
        query: SqlStr,
        // MSSQL always sends parameterized queries via tiberius (no server-side
        // prepared statement caching), so the persistent flag has no effect.
        _persistent: bool,
        arguments: Option<AnyArguments>,
    ) -> BoxStream<'_, sqlx_core::Result<Either<AnyQueryResult, AnyRow>>> {
        let arguments = match arguments.map(AnyArguments::convert_into).transpose() {
            Ok(arguments) => arguments,
            Err(error) => {
                return stream::once(future::ready(Err(sqlx_core::Error::Encode(error)))).boxed()
            }
        };

        Box::pin(
            stream::once(async move {
                let results = self.run(query.as_str(), arguments).await?;
                Ok::<_, sqlx_core::Error>(results)
            })
            .map_ok(|results| {
                futures_util::stream::iter(results.into_iter().map(|res| {
                    Ok(match res {
                        Either::Left(result) => Either::Left(map_result(result)),
                        Either::Right(row) => Either::Right(AnyRow::try_from(&row)?),
                    })
                }))
            })
            .try_flatten(),
        )
    }

    fn fetch_optional(
        &mut self,
        query: SqlStr,
        // See fetch_many: MSSQL has no server-side prepared statement caching.
        _persistent: bool,
        arguments: Option<AnyArguments>,
    ) -> BoxFuture<'_, sqlx_core::Result<Option<AnyRow>>> {
        let arguments = arguments
            .map(AnyArguments::convert_into)
            .transpose()
            .map_err(sqlx_core::Error::Encode);

        Box::pin(async move {
            let arguments = arguments?;
            let results = self.run(query.as_str(), arguments).await?;

            for result in results {
                if let Either::Right(row) = result {
                    return Ok(Some(AnyRow::try_from(&row)?));
                }
            }

            Ok(None)
        })
    }

    fn prepare_with<'c, 'q: 'c>(
        &'c mut self,
        sql: SqlStr,
        _parameters: &[AnyTypeInfo],
    ) -> BoxFuture<'c, sqlx_core::Result<AnyStatement>> {
        Box::pin(async move {
            let statement = Executor::prepare_with(self, sql, &[]).await?;
            let column_names = statement.metadata.column_names.clone();
            AnyStatement::try_from_statement(statement, column_names)
        })
    }

    #[cfg(feature = "offline")]
    fn describe(
        &mut self,
        sql: SqlStr,
    ) -> BoxFuture<'_, sqlx_core::Result<sqlx_core::describe::Describe<sqlx_core::any::Any>>> {
        Box::pin(async move {
            let describe = Executor::describe(self, sql).await?;
            describe.try_into_any()
        })
    }
}

impl<'a> TryFrom<&'a MssqlTypeInfo> for AnyTypeInfo {
    type Error = sqlx_core::Error;

    fn try_from(type_info: &'a MssqlTypeInfo) -> Result<Self, Self::Error> {
        Ok(AnyTypeInfo {
            kind: match type_info.base_name() {
                "TINYINT" => AnyTypeInfoKind::SmallInt,
                "SMALLINT" => AnyTypeInfoKind::SmallInt,
                "INT" => AnyTypeInfoKind::Integer,
                "BIGINT" => AnyTypeInfoKind::BigInt,
                "REAL" => AnyTypeInfoKind::Real,
                "FLOAT" => AnyTypeInfoKind::Double,
                "VARBINARY" | "BINARY" | "IMAGE" => AnyTypeInfoKind::Blob,
                "NULL" => AnyTypeInfoKind::Null,
                "BIT" => AnyTypeInfoKind::Bool,
                "MONEY" => AnyTypeInfoKind::Double,
                "SMALLMONEY" => AnyTypeInfoKind::Real,
                "DECIMAL" | "NUMERIC" => AnyTypeInfoKind::Text,
                "NVARCHAR" | "VARCHAR" | "NCHAR" | "CHAR" | "NTEXT" | "TEXT" | "XML" => {
                    AnyTypeInfoKind::Text
                }
                "UNIQUEIDENTIFIER" => AnyTypeInfoKind::Text,
                "DATE" | "TIME" | "DATETIME" | "DATETIME2" | "SMALLDATETIME" | "DATETIMEOFFSET" => {
                    AnyTypeInfoKind::Text
                }
                _ => {
                    return Err(sqlx_core::Error::AnyDriverError(
                        format!("Any driver does not support MSSQL type {type_info:?}").into(),
                    ))
                }
            },
        })
    }
}

impl<'a> TryFrom<&'a MssqlColumn> for AnyColumn {
    type Error = sqlx_core::Error;

    fn try_from(column: &'a MssqlColumn) -> Result<Self, Self::Error> {
        let type_info = AnyTypeInfo::try_from(&column.type_info)?;

        Ok(AnyColumn {
            ordinal: column.ordinal,
            name: column.name.clone(),
            type_info,
        })
    }
}

impl<'a> TryFrom<&'a MssqlRow> for AnyRow {
    type Error = sqlx_core::Error;

    fn try_from(row: &'a MssqlRow) -> Result<Self, Self::Error> {
        AnyRow::map_from(row, row.column_names.clone())
    }
}

impl<'a> TryFrom<&'a AnyConnectOptions> for MssqlConnectOptions {
    type Error = sqlx_core::Error;

    fn try_from(any_opts: &'a AnyConnectOptions) -> Result<Self, Self::Error> {
        let mut opts = Self::parse_from_url(&any_opts.database_url)?;
        opts.log_settings = any_opts.log_settings.clone();
        Ok(opts)
    }
}

fn map_result(result: MssqlQueryResult) -> AnyQueryResult {
    AnyQueryResult {
        rows_affected: result.rows_affected,
        last_insert_id: None,
    }
}
