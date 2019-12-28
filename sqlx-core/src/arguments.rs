//! Traits for passing arguments to SQL queries.

use crate::database::Database;
use crate::encode::Encode;
use crate::types::HasSqlType;

/// A tuple of arguments to be sent to the database.
pub trait Arguments: Send + Sized + Default + 'static {
    type Database: Database + ?Sized;

    /// Returns `true` if there are no values.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of values.
    fn len(&self) -> usize;

    /// Returns the size of the arguments, in bytes.
    fn size(&self) -> usize;

    /// Reserves the capacity for at least `len` more values (of `size` bytes) to
    /// be added to the arguments without a reallocation.  
    fn reserve(&mut self, len: usize, size: usize);

    /// Add the value to the end of the arguments.
    fn add<T>(&mut self, value: T)
    where
        Self::Database: HasSqlType<T>,
        T: Encode<Self::Database>;
}

pub trait IntoArguments<DB>
where
    DB: Database,
{
    fn into_arguments(self) -> DB::Arguments;
}

impl<DB> IntoArguments<DB> for DB::Arguments
where
    DB: Database,
{
    #[inline]
    fn into_arguments(self) -> DB::Arguments {
        self
    }
}

#[allow(unused)]
macro_rules! impl_into_arguments {
    ($B:ident: $( ($idx:tt) -> $T:ident );+;) => {
        impl<$($T,)+> crate::arguments::IntoArguments<$B> for ($($T,)+)
        where
            $($B: crate::types::HasSqlType<$T>,)+
            $($T: crate::encode::Encode<$B>,)+
        {
            fn into_arguments(self) -> <$B as crate::database::Database>::Arguments {
                use crate::arguments::Arguments;

                let mut arguments = <$B as crate::database::Database>::Arguments::default();

                let binds = 0 $(+ { $idx; 1 } )+;
                let bytes = 0 $(+ crate::encode::Encode::size_hint(&self.$idx))+;

                arguments.reserve(binds, bytes);

                $(crate::arguments::Arguments::bind(&mut arguments, self.$idx);)+

                arguments
            }
        }
    };
}

#[allow(unused)]
macro_rules! impl_into_arguments_for_database {
    ($B:ident) => {
        impl crate::arguments::IntoArguments<$B> for ()
        {
            #[inline]
            fn into_arguments(self) -> <$B as crate::database::Database>::Arguments {
                Default::default()
            }
        }

        impl_into_arguments!($B:
            (0) -> T1;
        );

        impl_into_arguments!($B:
            (0) -> T1;
            (1) -> T2;
        );

        impl_into_arguments!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
        );

        impl_into_arguments!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
        );

        impl_into_arguments!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
        );

        impl_into_arguments!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
        );

        impl_into_arguments!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
        );

        impl_into_arguments!($B:
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
            (7) -> T8;
        );

        impl_into_arguments!($B:
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
