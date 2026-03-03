use crate::database::MssqlArgumentValue;
use crate::error::{tiberius_err, Error};
use crate::executor::{Execute, Executor};
use crate::ext::ustr::UStr;
use crate::logger::QueryLogger;
use crate::statement::{MssqlStatement, MssqlStatementMetadata};
use crate::type_info::{type_name_for_tiberius, MssqlTypeInfo};
use crate::value::{column_data_to_mssql_data, MssqlData};
use crate::HashMap;
use crate::{
    Mssql, MssqlArguments, MssqlColumn, MssqlConnection, MssqlQueryResult, MssqlRow,
};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use sqlx_core::column::{ColumnOrigin, TableColumn};
use sqlx_core::sql_str::{AssertSqlSafe, SqlSafeStr, SqlStr};
use std::sync::Arc;

/// Newtype wrapper to bridge `tiberius::ColumnData` into `tiberius::IntoSql`.
///
/// tiberius implements `ToSql` but not `IntoSql` for some types (e.g. `time`
/// crate types, and `BigDecimal` due to version mismatch). `Query::bind()`
/// requires `IntoSql`, so this wrapper lets us construct `ColumnData` manually
/// and pass it to `bind()`.
#[cfg(any(feature = "chrono", feature = "time", feature = "bigdecimal"))]
struct ColumnDataWrapper<'a>(tiberius::ColumnData<'a>);

#[cfg(any(feature = "chrono", feature = "time", feature = "bigdecimal"))]
impl<'a> tiberius::IntoSql<'a> for ColumnDataWrapper<'a> {
    fn into_sql(self) -> tiberius::ColumnData<'a> {
        self.0
    }
}

impl MssqlConnection {
    /// Execute a query, eagerly collecting all results.
    ///
    /// We collect eagerly because `tiberius::QueryStream` borrows `&mut Client`,
    /// which prevents us from holding it across yield points alongside `&mut self`.
    pub(crate) async fn run(
        &mut self,
        sql: &str,
        arguments: Option<MssqlArguments>,
    ) -> Result<Vec<Either<MssqlQueryResult, MssqlRow>>, Error> {
        // Resolve any pending rollback first
        crate::transaction::resolve_pending_rollback(self).await?;

        let mut logger = QueryLogger::new(
            AssertSqlSafe(sql).into_sql_str(),
            self.inner.log_settings.clone(),
        );

        let mut results = Vec::new();

        if let Some(args) = arguments {
            // Parameterized query using tiberius::Query
            let mut query = tiberius::Query::new(sql);

            for arg in &args.values {
                match arg {
                    MssqlArgumentValue::Null => {
                        query.bind(Option::<&str>::None);
                    }
                    MssqlArgumentValue::Bool(v) => {
                        query.bind(*v);
                    }
                    MssqlArgumentValue::U8(v) => {
                        query.bind(*v);
                    }
                    MssqlArgumentValue::I16(v) => {
                        query.bind(*v);
                    }
                    MssqlArgumentValue::I32(v) => {
                        query.bind(*v);
                    }
                    MssqlArgumentValue::I64(v) => {
                        query.bind(*v);
                    }
                    MssqlArgumentValue::F32(v) => {
                        query.bind(*v);
                    }
                    MssqlArgumentValue::F64(v) => {
                        query.bind(*v);
                    }
                    MssqlArgumentValue::String(v) => {
                        query.bind(v.as_str());
                    }
                    MssqlArgumentValue::Binary(v) => {
                        query.bind(v.as_slice());
                    }
                    #[cfg(feature = "chrono")]
                    MssqlArgumentValue::NaiveDateTime(v) => {
                        query.bind(*v);
                    }
                    #[cfg(feature = "chrono")]
                    MssqlArgumentValue::NaiveDate(v) => {
                        query.bind(*v);
                    }
                    #[cfg(feature = "chrono")]
                    MssqlArgumentValue::NaiveTime(v) => {
                        query.bind(*v);
                    }
                    #[cfg(feature = "chrono")]
                    MssqlArgumentValue::DateTimeFixedOffset(v) => {
                        use chrono::Timelike as _;
                        let epoch = chrono::NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
                        let naive = v.naive_local();
                        let days = (naive.date() - epoch).num_days() as u32;
                        let total_ns = naive.time().num_seconds_from_midnight() as u64
                            * 1_000_000_000
                            + (naive.time().nanosecond() as u64 % 1_000_000_000);
                        let increments = total_ns / 100;
                        let offset_minutes =
                            v.offset().local_minus_utc() / 60;
                        let dt2 = tiberius::time::DateTime2::new(
                            tiberius::time::Date::new(days),
                            tiberius::time::Time::new(increments, 7),
                        );
                        let cd = tiberius::ColumnData::DateTimeOffset(Some(
                            tiberius::time::DateTimeOffset::new(
                                dt2,
                                offset_minutes as i16,
                            ),
                        ));
                        query.bind(ColumnDataWrapper(cd));
                    }
                    #[cfg(feature = "uuid")]
                    MssqlArgumentValue::Uuid(v) => {
                        query.bind(v);
                    }
                    #[cfg(feature = "rust_decimal")]
                    MssqlArgumentValue::Decimal(v) => {
                        let unpacked = v.unpack();
                        let mut value = (((unpacked.hi as u128) << 64)
                            + ((unpacked.mid as u128) << 32)
                            + unpacked.lo as u128)
                            as i128;
                        if v.is_sign_negative() {
                            value = -value;
                        }
                        query.bind(tiberius::numeric::Numeric::new_with_scale(
                            value,
                            v.scale() as u8,
                        ));
                    }
                    #[cfg(feature = "time")]
                    MssqlArgumentValue::TimeDate(v) => {
                        let epoch = time::Date::from_ordinal_date(1, 1).unwrap();
                        let days = (*v - epoch).whole_days() as u32;
                        let cd = tiberius::ColumnData::Date(Some(
                            tiberius::time::Date::new(days),
                        ));
                        query.bind(ColumnDataWrapper(cd));
                    }
                    #[cfg(feature = "time")]
                    MssqlArgumentValue::TimeTime(v) => {
                        let (h, m, s, ns) = v.as_hms_nano();
                        let total_ns = h as u64 * 3_600_000_000_000
                            + m as u64 * 60_000_000_000
                            + s as u64 * 1_000_000_000
                            + ns as u64;
                        // Scale 7 = 100ns increments
                        let increments = total_ns / 100;
                        let cd = tiberius::ColumnData::Time(Some(
                            tiberius::time::Time::new(increments, 7),
                        ));
                        query.bind(ColumnDataWrapper(cd));
                    }
                    #[cfg(feature = "time")]
                    MssqlArgumentValue::TimePrimitiveDateTime(v) => {
                        let date = v.date();
                        let time = v.time();
                        let epoch = time::Date::from_ordinal_date(1, 1).unwrap();
                        let days = (date - epoch).whole_days() as u32;
                        let (h, m, s, ns) = time.as_hms_nano();
                        let total_ns = h as u64 * 3_600_000_000_000
                            + m as u64 * 60_000_000_000
                            + s as u64 * 1_000_000_000
                            + ns as u64;
                        let increments = total_ns / 100;
                        let cd = tiberius::ColumnData::DateTime2(Some(
                            tiberius::time::DateTime2::new(
                                tiberius::time::Date::new(days),
                                tiberius::time::Time::new(increments, 7),
                            ),
                        ));
                        query.bind(ColumnDataWrapper(cd));
                    }
                    #[cfg(feature = "time")]
                    MssqlArgumentValue::TimeOffsetDateTime(v) => {
                        let epoch = time::Date::from_ordinal_date(1, 1).unwrap();
                        let offset_minutes = v.offset().whole_seconds() / 60;
                        let date = v.date();
                        let time = v.time();
                        let days = (date - epoch).whole_days() as u32;
                        let (h, m, s, ns) = time.as_hms_nano();
                        let total_ns = h as u64 * 3_600_000_000_000
                            + m as u64 * 60_000_000_000
                            + s as u64 * 1_000_000_000
                            + ns as u64;
                        let increments = total_ns / 100;
                        let dt2 = tiberius::time::DateTime2::new(
                            tiberius::time::Date::new(days),
                            tiberius::time::Time::new(increments, 7),
                        );
                        let cd = tiberius::ColumnData::DateTimeOffset(Some(
                            tiberius::time::DateTimeOffset::new(
                                dt2,
                                offset_minutes as i16,
                            ),
                        ));
                        query.bind(ColumnDataWrapper(cd));
                    }
                    #[cfg(feature = "bigdecimal")]
                    MssqlArgumentValue::BigDecimal(v) => {
                        use bigdecimal::ToPrimitive;
                        // Convert BigDecimal to tiberius Numeric
                        let (bigint, exponent) = v.as_bigint_and_exponent();
                        let scale = exponent.max(0) as u8;
                        // Convert to i128 for Numeric
                        let value: i128 = bigint.to_i128().ok_or_else(|| {
                            Error::Encode(
                                format!("BigDecimal value too large for SQL NUMERIC: {v}").into(),
                            )
                        })?;
                        let cd = tiberius::ColumnData::Numeric(Some(
                            tiberius::numeric::Numeric::new_with_scale(value, scale),
                        ));
                        query.bind(ColumnDataWrapper(cd));
                    }
                }
            }

            let stream = query.query(&mut self.inner.client).await.map_err(tiberius_err)?;
            collect_results(stream, &mut results, &mut logger).await?;
        } else {
            // Simple query (no parameters)
            let stream = self
                .inner
                .client
                .simple_query(sql)
                .await
                .map_err(tiberius_err)?;
            collect_results(stream, &mut results, &mut logger).await?;
        }

        Ok(results)
    }
}

/// Collect all results from a tiberius QueryStream into a Vec.
async fn collect_results<'a>(
    mut stream: tiberius::QueryStream<'a>,
    results: &mut Vec<Either<MssqlQueryResult, MssqlRow>>,
    logger: &mut QueryLogger,
) -> Result<(), Error> {
    // Process all result sets
    let mut columns: Option<Arc<Vec<MssqlColumn>>> = None;
    let mut column_names: Option<Arc<HashMap<UStr, usize>>> = None;
    let mut rows_affected: u64 = 0;

    while let Some(item) = stream.try_next().await.map_err(tiberius_err)? {
        match item {
            tiberius::QueryItem::Metadata(meta) => {
                // Build column info from metadata
                let cols: Vec<MssqlColumn> = meta
                    .columns()
                    .iter()
                    .enumerate()
                    .map(|(ordinal, col)| {
                        let name = UStr::new(col.name());
                        let type_info =
                            MssqlTypeInfo::new(type_name_for_tiberius(&col.column_type()));
                        MssqlColumn {
                            ordinal,
                            name,
                            type_info,
                            origin: ColumnOrigin::Unknown,
                        }
                    })
                    .collect();

                let names: HashMap<UStr, usize> = cols
                    .iter()
                    .enumerate()
                    .map(|(i, col)| (col.name.clone(), i))
                    .collect();

                columns = Some(Arc::new(cols));
                column_names = Some(Arc::new(names));
            }
            tiberius::QueryItem::Row(row) => {
                let cols = columns.as_ref().ok_or_else(|| {
                    Error::Protocol("row received before metadata".into())
                })?;
                let names = column_names.as_ref().ok_or_else(|| {
                    Error::Protocol("row received before metadata".into())
                })?;

                // Convert tiberius row to MssqlRow by iterating over cells
                let values: Vec<MssqlData> = row
                    .into_iter()
                    .map(|data| column_data_to_mssql_data(&data))
                    .collect();

                rows_affected += 1;
                logger.increment_rows_returned();
                results.push(Either::Right(MssqlRow {
                    values,
                    columns: Arc::clone(cols),
                    column_names: Arc::clone(names),
                }));
            }
        }
    }

    // Report query result with total rows
    logger.increase_rows_affected(rows_affected);
    results.push(Either::Left(MssqlQueryResult { rows_affected }));

    Ok(())
}

impl<'c> Executor<'c> for &'c mut MssqlConnection {
    type Database = Mssql;

    fn fetch_many<'e, 'q, E>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<MssqlQueryResult, MssqlRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
        'q: 'e,
        E: 'q,
    {
        let arguments = query.take_arguments().map_err(Error::Encode);
        // MSSQL always sends parameterized queries via tiberius — there is no
        // server-side prepared statement caching like PostgreSQL's, so this
        // flag is intentionally unused.
        let _persistent = query.persistent();
        let sql = query.sql();

        Box::pin(futures_util::stream::once(async move {
            let arguments = arguments?;
            let results = self.run(sql.as_str(), arguments).await?;
            Ok::<_, Error>(results)
        })
        .map_ok(|results| futures_util::stream::iter(results.into_iter().map(Ok)))
        .try_flatten())
    }

    fn fetch_optional<'e, 'q, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<MssqlRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
        'q: 'e,
        E: 'q,
    {
        let mut s = self.fetch_many(query);

        Box::pin(async move {
            while let Some(v) = s.try_next().await? {
                if let Either::Right(r) = v {
                    return Ok(Some(r));
                }
            }

            Ok(None)
        })
    }

    fn prepare_with<'e>(
        self,
        sql: SqlStr,
        _parameters: &'e [MssqlTypeInfo],
    ) -> BoxFuture<'e, Result<MssqlStatement, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            // Use sp_describe_first_result_set to get column metadata
            let describe_sql = format!(
                "EXEC sp_describe_first_result_set @tsql = N'{}'",
                sql.as_str().replace('\'', "''")
            );

            let mut columns = Vec::new();
            let mut column_names = HashMap::new();

            let stream = self
                .inner
                .client
                .simple_query(&describe_sql)
                .await
                .map_err(tiberius_err)?;

            let rows: Vec<tiberius::Row> = stream.into_first_result().await.map_err(tiberius_err)?;

            for (ordinal, row) in rows.iter().enumerate() {
                let name: &str = row.get("name").unwrap_or("");
                let type_name: &str = row.get("system_type_name").unwrap_or("UNKNOWN");
                let type_info = MssqlTypeInfo::new(type_name.to_uppercase());

                let source_table: Option<&str> = row.get("source_table");
                let source_schema: Option<&str> = row.get("source_schema");
                let source_column: Option<&str> = row.get("source_column");

                let origin = match (source_table, source_column) {
                    (Some(table), Some(col)) if !table.is_empty() && !col.is_empty() => {
                        let table_str = match source_schema {
                            Some(s) if !s.is_empty() => format!("{s}.{table}"),
                            _ => table.to_string(),
                        };
                        ColumnOrigin::Table(TableColumn {
                            table: table_str.into(),
                            name: col.into(),
                        })
                    }
                    _ => ColumnOrigin::Expression,
                };

                let ustr_name = UStr::new(name);
                column_names.insert(ustr_name.clone(), ordinal);
                columns.push(MssqlColumn {
                    ordinal,
                    name: ustr_name,
                    type_info,
                    origin,
                });
            }

            Ok(MssqlStatement {
                sql,
                metadata: MssqlStatementMetadata {
                    columns: Arc::new(columns),
                    column_names: Arc::new(column_names),
                    parameters: 0,
                },
            })
        })
    }

    #[doc(hidden)]
    #[cfg(feature = "offline")]
    fn describe<'e>(
        self,
        sql: SqlStr,
    ) -> BoxFuture<'e, Result<crate::describe::Describe<Mssql>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            // Query sp_describe_first_result_set directly so we can extract nullable info
            let describe_sql = format!(
                "EXEC sp_describe_first_result_set @tsql = N'{}'",
                sql.as_str().replace('\'', "''")
            );

            let stream = self
                .inner
                .client
                .simple_query(&describe_sql)
                .await
                .map_err(tiberius_err)?;

            let rows: Vec<tiberius::Row> =
                stream.into_first_result().await.map_err(tiberius_err)?;

            let mut columns = Vec::new();
            let mut column_names = HashMap::new();
            let mut nullable = Vec::new();

            for (ordinal, row) in rows.iter().enumerate() {
                let name: &str = row.get("name").unwrap_or("");
                let type_name: &str = row.get("system_type_name").unwrap_or("UNKNOWN");
                let type_info = MssqlTypeInfo::new(type_name.to_uppercase());
                let is_nullable: Option<bool> = row.get("is_nullable");

                let source_table: Option<&str> = row.get("source_table");
                let source_schema: Option<&str> = row.get("source_schema");
                let source_column: Option<&str> = row.get("source_column");

                let origin = match (source_table, source_column) {
                    (Some(table), Some(col)) if !table.is_empty() && !col.is_empty() => {
                        let table_str = match source_schema {
                            Some(s) if !s.is_empty() => format!("{s}.{table}"),
                            _ => table.to_string(),
                        };
                        ColumnOrigin::Table(TableColumn {
                            table: table_str.into(),
                            name: col.into(),
                        })
                    }
                    _ => ColumnOrigin::Expression,
                };

                let ustr_name = UStr::new(name);
                column_names.insert(ustr_name.clone(), ordinal);
                columns.push(MssqlColumn {
                    ordinal,
                    name: ustr_name,
                    type_info,
                    origin,
                });
                nullable.push(is_nullable);
            }

            // Count parameters using sp_describe_undeclared_parameters
            let param_sql = format!(
                "EXEC sp_describe_undeclared_parameters @tsql = N'{}'",
                sql.as_str().replace('\'', "''")
            );
            let param_count = match self
                .inner
                .client
                .simple_query(&param_sql)
                .await
            {
                Ok(stream) => stream
                    .into_first_result()
                    .await
                    .map_err(tiberius_err)?
                    .len(),
                Err(_) => 0,
            };

            Ok(crate::describe::Describe {
                parameters: Some(Either::Right(param_count)),
                columns,
                nullable,
            })
        })
    }
}
