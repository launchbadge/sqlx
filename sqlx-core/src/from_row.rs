use crate::{Row, TypeDecodeOwned};

/// A type that can be built from a row returned by the database.
pub trait FromRow<R: Row>: Sized {
    /// Build a value of `Self` from the given [`Row`].
    fn from_row(row: &R) -> crate::Result<Self>;
}

macro_rules! impl_from_row_for_tuple {
    ($( ($idx:tt) -> $T:ident );+;) => {
        impl<R: Row, $($T,)+> FromRow<R> for ($($T,)+)
        where
            $($T: TypeDecodeOwned<R::Database>,)+
        {
            fn from_row(row: &R) -> crate::Result<Self> {
                Ok(($(row.try_get($idx)?,)+))
            }
        }
    };
}

impl_from_row_for_tuple!(
    (0) -> T1;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
);

impl_from_row_for_tuple!(
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

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
    (12) -> T13;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
    (12) -> T13;
    (13) -> T14;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
    (12) -> T13;
    (13) -> T14;
    (14) -> T15;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
    (12) -> T13;
    (13) -> T14;
    (14) -> T15;
    (15) -> T16;
);
