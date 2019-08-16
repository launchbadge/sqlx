use crate::{backend::Backend, deserialize::FromSql, types::HasSqlType};

pub trait Row: Send {
    type Backend: Backend;

    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    fn get_raw(&self, index: usize) -> Option<&[u8]>;

    #[inline]
    fn get<ST, T>(&self, index: usize) -> T
    where
        Self::Backend: HasSqlType<ST>,
        T: FromSql<ST, Self::Backend>,
    {
        T::from_sql(self.get_raw(index))
    }
}

pub trait FromRow<A, DB: Backend> {
    fn from_row<R: Row<Backend = DB>>(row: R) -> Self;
}

impl<T, ST, DB> FromRow<ST, DB> for T
where
    DB: Backend + HasSqlType<ST>,
    T: FromSql<ST, DB>,
{
    #[inline]
    fn from_row<R: Row<Backend = DB>>(row: R) -> Self {
        row.get::<ST, T>(0)
    }
}

#[allow(unused)]
macro_rules! impl_from_row_tuple {
    ($B:ident: $( ($idx:tt) -> $T:ident, $ST:ident );+;) => {
        impl<$($ST,)+ $($T,)+> crate::row::FromRow<($($ST,)+), $B> for ($($T,)+)
        where
            $($B: crate::types::HasSqlType<$ST>,)+
            $($T: crate::deserialize::FromSql<$ST, $B>,)+
        {
            #[inline]
            fn from_row<R: crate::row::Row<Backend = $B>>(row: R) -> Self {
                ($(row.get::<$ST, $T>($idx),)+)
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_from_row_tuples_for_backend {
    ($B:ident) => {
        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
        );

        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
            (1) -> ST2, T2;
        );

        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
            (1) -> ST2, T2;
            (2) -> ST3, T3;
        );

        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
            (1) -> ST2, T2;
            (2) -> ST3, T3;
            (3) -> ST4, T4;
        );

        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
            (1) -> ST2, T2;
            (2) -> ST3, T3;
            (3) -> ST4, T4;
            (4) -> ST5, T5;
        );

        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
            (1) -> ST2, T2;
            (2) -> ST3, T3;
            (3) -> ST4, T4;
            (4) -> ST5, T5;
            (5) -> ST6, T6;
        );

        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
            (1) -> ST2, T2;
            (2) -> ST3, T3;
            (3) -> ST4, T4;
            (4) -> ST5, T5;
            (5) -> ST6, T6;
            (6) -> ST7, T7;
        );

        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
            (1) -> ST2, T2;
            (2) -> ST3, T3;
            (3) -> ST4, T4;
            (4) -> ST5, T5;
            (5) -> ST6, T6;
            (6) -> ST7, T7;
            (7) -> ST8, T8;
        );

        impl_from_row_tuple!($B:
            (0) -> ST1, T1;
            (1) -> ST2, T2;
            (2) -> ST3, T3;
            (3) -> ST4, T4;
            (4) -> ST5, T5;
            (5) -> ST6, T6;
            (6) -> ST7, T7;
            (7) -> ST8, T8;
            (8) -> ST9, T9;
        );
    }
}
