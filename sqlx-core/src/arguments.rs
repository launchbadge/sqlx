use crate::database::Database;
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
