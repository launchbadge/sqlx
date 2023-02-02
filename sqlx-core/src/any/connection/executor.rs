use crate::any::{
    Any, AnyColumn, AnyConnection, AnyQueryResult, AnyRow, AnyStatement, AnyTypeInfo,
};
use crate::database::Database;
use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{StreamExt, TryStreamExt};

impl<'c> Executor<'c> for &'c mut AnyConnection {
    type Database = Any;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<AnyQueryResult, AnyRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Any>,
    {
        let arguments = query.take_arguments();
        self.backend.fetch_many(query.sql(), arguments)
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
        self.backend.fetch_optional(query.sql(), arguments)
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &[AnyTypeInfo],
    ) -> BoxFuture<'e, Result<AnyStatement<'q>, Error>>
    where
        'c: 'e,
    {
        self.backend.prepare_with(sql, parameters)
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        self.backend.describe(sql)
    }
}

fn map_describe<DB: Database>(info: Describe<DB>) -> Describe<Any>
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

    Describe {
        parameters,
        nullable: info.nullable,
        columns: info.columns.into_iter().map(Into::into).collect(),
    }
}
