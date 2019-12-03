use crate::{backend::Backend, decode::Decode, types::HasSqlType};

pub trait Row: Send {
    type Backend: Backend;

    fn len(&self) -> usize;

    fn get_raw(&self, index: usize) -> Option<&[u8]>;

    fn get<T>(&self, index: usize) -> T
    where
        Self::Backend: HasSqlType<T>,
        T: Decode<Self::Backend>,
    {
        T::decode(self.get_raw(index))
    }
}

pub trait FromRow<DB: Backend> {
    fn from_row(row: <DB as Backend>::Row) -> Self;
}

#[allow(unused)]
macro_rules! impl_from_row {
    ($B:ident: $( ($idx:tt) -> $T:ident );+;) => {
        // Row -> (T1, T2, ...)
        impl<$($T,)+> crate::row::FromRow<$B> for ($($T,)+)
        where
            $($B: crate::types::HasSqlType<$T>,)+
            $($T: crate::decode::Decode<$B>,)+
        {
            #[inline]
            fn from_row(row: <$B as crate::backend::Backend>::Row) -> Self {
                use crate::row::Row;

                ($(row.get($idx),)+)
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_from_row_for_backend {
    ($B:ident, $row:ident) => {
        impl crate::row::FromRow<$B> for $row where $B: crate::Backend {
            #[inline]
            fn from_row(row: <$B as crate::backend::Backend>::Row) -> Self {
                row
            }
        }

        impl_from_row!($B:
            (0) -> T1;
        );

        impl_from_row!($B:
            (0) -> T1;
            (1) -> T2;
        );

        impl_from_row!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
        );

        impl_from_row!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
        );

        impl_from_row!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
        );

        impl_from_row!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
        );

        impl_from_row!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
        );

        impl_from_row!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
            (7) -> T8;
        );

        impl_from_row!($B:
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
