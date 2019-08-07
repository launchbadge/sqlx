use crate::{backend::Backend, deserialize::FromSql, postgres::Postgres, types::SqlType};

pub trait RawRow {
    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    fn get(&self, index: usize) -> Option<&[u8]>;
}

pub struct Row<B>(pub(crate) B::RawRow)
where
    B: Backend;

impl<B> Row<B>
where
    B: Backend,
{
    #[inline]
    pub fn get<ST, T>(&self, index: usize) -> T
    where
        ST: SqlType<B>,
        T: FromSql<B, ST>,
    {
        T::from_sql(self.0.get(index))
    }
}

pub trait FromRow<B, Record>
where
    B: Backend,
{
    fn from_row(row: Row<B>) -> Self;
}

impl<B, ST, T> FromRow<B, ST> for T
where
    B: Backend,
    ST: SqlType<B>,
    T: FromSql<B, ST>,
{
    #[inline]
    fn from_row(row: Row<B>) -> Self {
        row.get::<ST, T>(0)
    }
}

// TODO: Think of a better way to generate these tuple implementations

macro_rules! impl_from_row_tuple {
    ($B:ident: $( ($idx:tt) -> $T:ident, $ST:ident );+;) => {
        impl<$($ST,)+ $($T,)+> FromRow<Postgres, ($($ST,)+)> for ($($T,)+)
        where
            $($ST: SqlType<Postgres>,)+
            $($T: FromSql<Postgres, $ST>,)+
        {
            #[inline]
            fn from_row(row: Row<$B>) -> Self {
                ($(row.get::<$ST, $T>($idx),)+)
            }
        }
    };
}

impl_from_row_tuple!(Postgres:
    (0) -> ST1, T1;
);

impl_from_row_tuple!(Postgres:
    (0) -> ST1, T1;
    (1) -> ST2, T2;
);

impl_from_row_tuple!(Postgres:
    (0) -> ST1, T1;
    (1) -> ST2, T2;
    (2) -> ST3, T3;
);

impl_from_row_tuple!(Postgres:
    (0) -> ST1, T1;
    (1) -> ST2, T2;
    (2) -> ST3, T3;
    (3) -> ST4, T4;
);

impl_from_row_tuple!(Postgres:
    (0) -> ST1, T1;
    (1) -> ST2, T2;
    (2) -> ST3, T3;
    (3) -> ST4, T4;
    (4) -> ST5, T5;
);

impl_from_row_tuple!(Postgres:
    (0) -> ST1, T1;
    (1) -> ST2, T2;
    (2) -> ST3, T3;
    (3) -> ST4, T4;
    (4) -> ST5, T5;
    (5) -> ST6, T6;
);

impl_from_row_tuple!(Postgres:
    (0) -> ST1, T1;
    (1) -> ST2, T2;
    (2) -> ST3, T3;
    (3) -> ST4, T4;
    (4) -> ST5, T5;
    (5) -> ST6, T6;
    (6) -> ST7, T7;
);

impl_from_row_tuple!(Postgres:
    (0) -> ST1, T1;
    (1) -> ST2, T2;
    (2) -> ST3, T3;
    (3) -> ST4, T4;
    (4) -> ST5, T5;
    (5) -> ST6, T6;
    (6) -> ST7, T7;
    (7) -> ST8, T8;
);

impl_from_row_tuple!(Postgres:
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
