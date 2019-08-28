use crate::{
    backend::Backend, error::Error, executor::Executor, row::FromSqlRow, serialize::ToSql,
    types::HasSqlType,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait QueryParameters: Send {
    type Backend: Backend;

    fn new() -> Self
    where
        Self: Sized;

    fn bind<T>(&mut self, value: T)
    where
        Self::Backend: HasSqlType<T>,
        T: ToSql<Self::Backend>;
}

pub trait IntoQueryParameters<DB>
where
    DB: Backend,
{
    fn into(self) -> DB::QueryParameters;
}

#[allow(unused)]
macro_rules! impl_into_query_parameters {
    ($B:ident: $( ($idx:tt) -> $T:ident );+;) => {
        impl<$($T,)+> crate::query::IntoQueryParameters<$B> for ($($T,)+)
        where
            $($B: crate::types::HasSqlType<$T>,)+
            $($T: crate::serialize::ToSql<$B>,)+
        {
            fn into(self) -> <$B as crate::backend::Backend>::QueryParameters {
                let mut params = <<$B as crate::backend::Backend>::QueryParameters
                    as crate::query::QueryParameters>::new();

                $(crate::query::QueryParameters::bind(&mut params, self.$idx);)+

                params
            }
        }
    };
}

impl<DB> IntoQueryParameters<DB> for DB::QueryParameters
where
    DB: Backend,
{
    #[inline]
    fn into(self) -> DB::QueryParameters {
        self
    }
}

#[allow(unused)]
macro_rules! impl_into_query_parameters_for_backend {
    ($B:ident) => {
        impl crate::query::IntoQueryParameters<$B> for ()
        {
            #[inline]
            fn into(self) -> <$B as crate::backend::Backend>::QueryParameters {
                <<$B as crate::backend::Backend>::QueryParameters
                    as crate::query::QueryParameters>::new()
            }
        }

        impl_into_query_parameters!($B:
            (0) -> T1;
        );

        impl_into_query_parameters!($B:
            (0) -> T1;
            (1) -> T2;
        );

        impl_into_query_parameters!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
        );

        impl_into_query_parameters!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
        );

        impl_into_query_parameters!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
        );

        impl_into_query_parameters!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
        );

        impl_into_query_parameters!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
        );

        impl_into_query_parameters!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
            (7) -> T8;
        );

        impl_into_query_parameters!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
            (7) -> T8;
            (8) -> T9;
        );
    }
}
