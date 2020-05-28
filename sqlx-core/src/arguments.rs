//! Types and traits for passing arguments to SQL queries.

use crate::database::{Database, HasArguments};
use crate::encode::Encode;

/// A tuple of arguments to be sent to the database.
pub trait Arguments<'q>: Send + Sized + Default {
    type Database: Database;

    /// Reserves the capacity for at least `additional` more values (of `size` total bytes) to
    /// be added to the arguments without a reallocation.
    fn reserve(&mut self, additional: usize, size: usize);

    /// Add the value to the end of the arguments.
    fn add<T>(&mut self, value: T)
    where
        T: 'q + Encode<'q, Self::Database>;
}

pub trait IntoArguments<'q, DB: HasArguments<'q>>: Sized + Send {
    fn into_arguments(self) -> <DB as HasArguments<'q>>::Arguments;
}

// NOTE: required due to lack of lazy normalization
macro_rules! impl_into_arguments_for_arguments {
    ($Arguments:path) => {
        impl<'q>
            crate::arguments::IntoArguments<
                'q,
                <$Arguments as crate::arguments::Arguments<'q>>::Database,
            > for $Arguments
        {
            fn into_arguments(self) -> $Arguments {
                self
            }
        }
    };
}

// TODO: Impl `IntoArguments` for &[&dyn Encode]
// TODO: Impl `IntoArguments` for (impl Encode, ...) x16
