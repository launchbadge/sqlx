use crate::{backend::Backend, deserialize::FromSql, types::HasSqlType};

pub trait Row: Send {
    type Backend: Backend;

    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    fn get_raw(&self, index: usize) -> Option<&[u8]>;

    #[inline]
    fn get<T>(&self, index: usize) -> T
    where
        Self::Backend: HasSqlType<T>,
        T: FromSql<Self::Backend>,
    {
        T::from_sql(self.get_raw(index))
    }
}

pub trait FromSqlRow<DB: Backend> {
    fn from_row<R: Row<Backend = DB>>(row: R) -> Self;
}

impl<T, DB> FromSqlRow<DB> for T
where
    DB: Backend + HasSqlType<T>,
    T: FromSql<DB>,
{
    #[inline]
    fn from_row<R: Row<Backend = DB>>(row: R) -> Self {
        row.get::<T>(0)
    }
}

#[allow(unused)]
macro_rules! impl_from_sql_row_tuple {
    ($B:ident: $( ($idx:tt) -> $T:ident );+;) => {
        impl<$($T,)+> crate::row::FromSqlRow<$B> for ($($T,)+)
        where
            $($B: crate::types::HasSqlType<$T>,)+
            $($T: crate::deserialize::FromSql<$B>,)+
        {
            #[inline]
            fn from_row<R: crate::row::Row<Backend = $B>>(row: R) -> Self {
                ($(row.get($idx),)+)
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_from_sql_row_tuples_for_backend {
    ($B:ident) => {
        impl_from_sql_row_tuple!($B:
            (0) -> T1;
        );

        impl_from_sql_row_tuple!($B:
            (0) -> T1;
            (1) -> T2;
        );

        impl_from_sql_row_tuple!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
        );

        impl_from_sql_row_tuple!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
        );

        impl_from_sql_row_tuple!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
        );

        impl_from_sql_row_tuple!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
        );

        impl_from_sql_row_tuple!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
        );

        impl_from_sql_row_tuple!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
            (7) -> T8;
        );

        impl_from_sql_row_tuple!($B:
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
