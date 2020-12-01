use super::AuroraConnection;
use crate::aurora::error::AuroraDatabaseError;
use crate::aurora::statement::AuroraStatementMetadata;
use crate::aurora::{
    Aurora, AuroraArguments, AuroraColumn, AuroraDone, AuroraRow, AuroraStatement, AuroraTypeInfo,
};
use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::ext::ustr::UStr;

use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::stream;
use futures_util::{pin_mut, TryStreamExt};
use rusoto_rds_data::{ExecuteStatementRequest, ExecuteStatementResponse, RdsData};
use std::borrow::Cow;
use std::sync::Arc;

impl AuroraConnection {
    async fn run<'e, 'c: 'e, 'q: 'e>(
        &'c mut self,
        query: &'q str,
        arguments: Option<AuroraArguments>,
    ) -> Result<impl Stream<Item = Result<Either<AuroraDone, AuroraRow>, Error>> + 'e, Error> {
        // TODO: is this correct?
        let transaction_id = self.transaction_ids.last().cloned();

        let request = ExecuteStatementRequest {
            sql: query.to_owned(),
            parameters: arguments.map(|m| m.parameters),
            resource_arn: self.resource_arn.clone(),
            secret_arn: self.secret_arn.clone(),
            database: self.database.clone(),
            schema: self.schema.clone(),
            transaction_id,
            include_result_metadata: Some(true),
            ..Default::default()
        };

        let ExecuteStatementResponse {
            column_metadata,
            number_of_records_updated,
            records,
            ..
        } = self
            .client
            .execute_statement(request)
            .await
            .map_err(AuroraDatabaseError)?;

        let rows_affected = number_of_records_updated.unwrap_or_default() as u64;
        let column_metadata = column_metadata.unwrap_or_default();

        let mut rows = records
            .unwrap_or_default()
            .into_iter()
            .map(|fields| {
                let columns: Vec<_> = fields
                    .iter()
                    .zip(&column_metadata)
                    .enumerate()
                    .map(|(ordinal, (field, metadata))| AuroraColumn {
                        ordinal,
                        name: UStr::new(metadata.name.as_deref().unwrap_or_default()),
                        type_info: AuroraTypeInfo::from(field),
                    })
                    .collect();

                let column_names = columns
                    .iter()
                    .map(|column| (column.name.clone(), column.ordinal))
                    .collect();
                let parameters = columns.iter().map(|column| column.type_info).collect();

                let metadata = Arc::new(AuroraStatementMetadata {
                    columns,
                    column_names,
                    parameters,
                });

                let row = AuroraRow { fields, metadata };

                Ok(Either::Right(row))
            })
            .collect::<Vec<_>>();

        rows.push(Ok(Either::Left(AuroraDone { rows_affected })));

        Ok(stream::iter(rows))
    }
}

impl<'c> Executor<'c> for &'c mut AuroraConnection {
    type Database = Aurora;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<AuroraDone, AuroraRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();

        // TODO: implement statement caching?
        //let metadata = query.statement();
        //let persistent = query.persistent();

        Box::pin(try_stream! {
            let s = self.run(sql, arguments).await?;
            pin_mut!(s);

            while let Some(v) = s.try_next().await? {
                r#yield!(v);
            }

            Ok(())
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<AuroraRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
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

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        _parameters: &[AuroraTypeInfo],
    ) -> BoxFuture<'e, Result<AuroraStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            Ok(AuroraStatement {
                sql: Cow::Borrowed(sql),
                metadata: Default::default(),
            })
        })
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            let metadata: AuroraStatementMetadata = Default::default();

            let nullable = Vec::with_capacity(metadata.columns.len());

            Ok(Describe {
                nullable,
                columns: metadata.columns,
                parameters: None,
            })
        })
    }
}
