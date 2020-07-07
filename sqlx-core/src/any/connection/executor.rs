use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{StreamExt, TryStreamExt};

use crate::any::connection::AnyConnectionKind;
use crate::any::{Any, AnyColumn, AnyConnection, AnyDone, AnyRow, AnyTypeInfo};
use crate::database::Database;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::statement::StatementInfo;

impl<'c> Executor<'c> for &'c mut AnyConnection {
    type Database = Any;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<AnyDone, AnyRow>, Error>>
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
                .map_ok(|v| v.map_right(Into::into).map_left(Into::into))
                .boxed(),

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => conn
                .fetch_many((query, arguments.map(Into::into)))
                .map_ok(|v| v.map_right(Into::into).map_left(Into::into))
                .boxed(),

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => conn
                .fetch_many((query, arguments.map(Into::into)))
                .map_ok(|v| v.map_right(Into::into).map_left(Into::into))
                .boxed(),

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => conn
                .fetch_many((query, arguments.map(Into::into)))
                .map_ok(|v| v.map_right(Into::into).map_left(Into::into))
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
                    .map(Into::into),

                #[cfg(feature = "mysql")]
                AnyConnectionKind::MySql(conn) => conn
                    .fetch_optional((query, arguments.map(Into::into)))
                    .await?
                    .map(Into::into),

                #[cfg(feature = "sqlite")]
                AnyConnectionKind::Sqlite(conn) => conn
                    .fetch_optional((query, arguments.map(Into::into)))
                    .await?
                    .map(Into::into),

                #[cfg(feature = "mssql")]
                AnyConnectionKind::Mssql(conn) => conn
                    .fetch_optional((query, arguments.map(Into::into)))
                    .await?
                    .map(Into::into),
            })
        })
    }

    fn describe<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<StatementInfo<Self::Database>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let query = query.query();

        Box::pin(async move {
            Ok(match &mut self.0 {
                #[cfg(feature = "postgres")]
                AnyConnectionKind::Postgres(conn) => {
                    conn.describe(query).await.map(map_describe)?
                }

                #[cfg(feature = "mysql")]
                AnyConnectionKind::MySql(conn) => conn.describe(query).await.map(map_describe)?,

                #[cfg(feature = "sqlite")]
                AnyConnectionKind::Sqlite(conn) => conn.describe(query).await.map(map_describe)?,

                #[cfg(feature = "mssql")]
                AnyConnectionKind::Mssql(conn) => conn.describe(query).await.map(map_describe)?,
            })
        })
    }

    fn describe_full<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<StatementInfo<Self::Database>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let query = query.query();

        Box::pin(async move {
            Ok(match &mut self.0 {
                #[cfg(feature = "postgres")]
                AnyConnectionKind::Postgres(conn) => {
                    conn.describe_full(query).await.map(map_describe)?
                }

                #[cfg(feature = "mysql")]
                AnyConnectionKind::MySql(conn) => {
                    conn.describe_full(query).await.map(map_describe)?
                }

                #[cfg(feature = "sqlite")]
                AnyConnectionKind::Sqlite(conn) => {
                    conn.describe_full(query).await.map(map_describe)?
                }

                #[cfg(feature = "mssql")]
                AnyConnectionKind::Mssql(conn) => {
                    conn.describe_full(query).await.map(map_describe)?
                }
            })
        })
    }
}

fn map_describe<DB: Database>(info: StatementInfo<DB>) -> StatementInfo<Any>
where
    AnyTypeInfo: From<DB::TypeInfo>,
    AnyColumn: From<DB::Column>,
{
    let parameters = match info.parameters {
        None => None,
        Some(Either::Right(num)) => Some(Either::Right(num)),
        Some(Either::Left(params)) => {
            Some(Either::Left(params.into_iter().map(Into::into).collect()))
        }
    };

    StatementInfo {
        parameters,
        nullable: info.nullable,
        columns: info.columns.into_iter().map(Into::into).collect(),
    }
}
