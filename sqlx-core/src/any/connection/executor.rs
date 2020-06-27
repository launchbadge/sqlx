use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{StreamExt, TryStreamExt};

use crate::any::connection::AnyConnectionKind;
use crate::any::row::AnyRowKind;
use crate::any::type_info::AnyTypeInfoKind;
use crate::any::{Any, AnyConnection, AnyRow, AnyTypeInfo};
use crate::describe::{Column, Describe};
use crate::error::Error;
use crate::executor::{Execute, Executor};

// FIXME: Some of the below, describe especially, is very messy/duplicated; perhaps we should have
//        an `Into` that goes from `PgTypeInfo` to `AnyTypeInfo` and so on

impl<'c> Executor<'c> for &'c mut AnyConnection {
    type Database = Any;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<u64, AnyRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let arguments = query.take_arguments();
        let query = query.query();

        match &mut self.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => conn
                .fetch_many((query, arguments.map(Into::into)))
                .map_ok(|v| match v {
                    Either::Right(row) => Either::Right(AnyRow(AnyRowKind::Postgres(row))),
                    Either::Left(count) => Either::Left(count),
                })
                .boxed(),

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => conn
                .fetch_many((query, arguments.map(Into::into)))
                .map_ok(|v| match v {
                    Either::Right(row) => Either::Right(AnyRow(AnyRowKind::MySql(row))),
                    Either::Left(count) => Either::Left(count),
                })
                .boxed(),

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => conn
                .fetch_many((query, arguments.map(Into::into)))
                .map_ok(|v| match v {
                    Either::Right(row) => Either::Right(AnyRow(AnyRowKind::Sqlite(row))),
                    Either::Left(count) => Either::Left(count),
                })
                .boxed(),

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => conn
                .fetch_many((query, arguments.map(Into::into)))
                .map_ok(|v| match v {
                    Either::Right(row) => Either::Right(AnyRow(AnyRowKind::Mssql(row))),
                    Either::Left(count) => Either::Left(count),
                })
                .boxed(),
        }
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxFuture<'e, Result<Option<AnyRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let arguments = query.take_arguments();
        let query = query.query();

        Box::pin(async move {
            Ok(match &mut self.0 {
                #[cfg(feature = "postgres")]
                AnyConnectionKind::Postgres(conn) => conn
                    .fetch_optional((query, arguments.map(Into::into)))
                    .await?
                    .map(AnyRowKind::Postgres),

                #[cfg(feature = "mysql")]
                AnyConnectionKind::MySql(conn) => conn
                    .fetch_optional((query, arguments.map(Into::into)))
                    .await?
                    .map(AnyRowKind::MySql),

                #[cfg(feature = "sqlite")]
                AnyConnectionKind::Sqlite(conn) => conn
                    .fetch_optional((query, arguments.map(Into::into)))
                    .await?
                    .map(AnyRowKind::Sqlite),

                #[cfg(feature = "mssql")]
                AnyConnectionKind::Mssql(conn) => conn
                    .fetch_optional((query, arguments.map(Into::into)))
                    .await?
                    .map(AnyRowKind::Mssql),
            }
            .map(AnyRow))
        })
    }

    fn describe<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let query = query.query();

        Box::pin(async move {
            Ok(match &mut self.0 {
                #[cfg(feature = "postgres")]
                AnyConnectionKind::Postgres(conn) => {
                    conn.describe(query).await.map(|desc| Describe {
                        params: desc
                            .params
                            .into_iter()
                            .map(|ty| ty.map(AnyTypeInfoKind::Postgres).map(AnyTypeInfo))
                            .collect(),

                        columns: desc
                            .columns
                            .into_iter()
                            .map(|column| Column {
                                name: column.name,
                                not_null: column.not_null,
                                type_info: column
                                    .type_info
                                    .map(AnyTypeInfoKind::Postgres)
                                    .map(AnyTypeInfo),
                            })
                            .collect(),
                    })?
                }

                #[cfg(feature = "mysql")]
                AnyConnectionKind::MySql(conn) => {
                    conn.describe(query).await.map(|desc| Describe {
                        params: desc
                            .params
                            .into_iter()
                            .map(|ty| ty.map(AnyTypeInfoKind::MySql).map(AnyTypeInfo))
                            .collect(),

                        columns: desc
                            .columns
                            .into_iter()
                            .map(|column| Column {
                                name: column.name,
                                not_null: column.not_null,
                                type_info: column
                                    .type_info
                                    .map(AnyTypeInfoKind::MySql)
                                    .map(AnyTypeInfo),
                            })
                            .collect(),
                    })?
                }

                #[cfg(feature = "sqlite")]
                AnyConnectionKind::Sqlite(conn) => {
                    conn.describe(query).await.map(|desc| Describe {
                        params: desc
                            .params
                            .into_iter()
                            .map(|ty| ty.map(AnyTypeInfoKind::Sqlite).map(AnyTypeInfo))
                            .collect(),

                        columns: desc
                            .columns
                            .into_iter()
                            .map(|column| Column {
                                name: column.name,
                                not_null: column.not_null,
                                type_info: column
                                    .type_info
                                    .map(AnyTypeInfoKind::Sqlite)
                                    .map(AnyTypeInfo),
                            })
                            .collect(),
                    })?
                }

                #[cfg(feature = "mssql")]
                AnyConnectionKind::Mssql(conn) => {
                    conn.describe(query).await.map(|desc| Describe {
                        params: desc
                            .params
                            .into_iter()
                            .map(|ty| ty.map(AnyTypeInfoKind::Mssql).map(AnyTypeInfo))
                            .collect(),

                        columns: desc
                            .columns
                            .into_iter()
                            .map(|column| Column {
                                name: column.name,
                                not_null: column.not_null,
                                type_info: column
                                    .type_info
                                    .map(AnyTypeInfoKind::Mssql)
                                    .map(AnyTypeInfo),
                            })
                            .collect(),
                    })?
                }
            })
        })
    }
}
