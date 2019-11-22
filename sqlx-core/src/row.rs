use crate::{backend::Backend, decode::Decode, types::HasSqlType};

pub trait RawRow: Send {
    type Backend: Backend;

    fn len(&self) -> usize;

    fn get_raw(&self, index: usize) -> Option<&[u8]>;
}

pub struct Row<DB>(pub(crate) DB::Row)
where
    DB: Backend;

impl<DB> Row<DB>
where
    DB: Backend,
{
    pub fn get<T>(&self, index: usize) -> T
    where
        DB: HasSqlType<T>,
        T: Decode<DB>,
    {
        T::decode(self.0.get_raw(index))
    }
}

pub trait FromRow<DB: Backend> {
    fn from_row(row: DB::Row) -> Self;
}

impl<T, DB> FromRow<DB> for T
where
    DB: Backend + HasSqlType<T>,
    T: Decode<DB>,
{
    #[inline]
    fn from_row(row: DB::Row) -> Self {
        T::decode(row.get_raw(0))
    }
}

#[allow(unused)]
macro_rules! impl_from_sql_row_tuple {
    ($B:ident: $( ($idx:tt) -> $T:ident );+;) => {
        impl<$($T,)+> crate::row::FromRow<$B> for ($($T,)+)
        where
            $($B: crate::types::HasSqlType<$T>,)+
            $($T: crate::decode::Decode<$B>,)+
        {
            #[inline]
            fn from_row(row: <$B as crate::Backend>::Row) -> Self {
                use crate::row::RawRow;

                ($($T::decode(row.get_raw($idx)),)+)
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_from_row_for_backend {
    ($B:ident) => {
        impl crate::row::FromRow<$B> for crate::row::Row<$B> where $B: crate::Backend {
            #[inline]
            fn from_row(row: <$B as crate::Backend>::Row) -> Self {
                Self(row)
            }
        }

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
