//! Traits for passing arguments to SQL queries.

use crate::database::Database;
use crate::encode::Encode;
use crate::types::Type;

/// A tuple of arguments to be sent to the database.
pub trait Arguments: Send + Sized + Default + 'static {
    type Database: Database + ?Sized;

    /// Reserves the capacity for at least `len` more values (of `size` bytes) to
    /// be added to the arguments without a reallocation.  
    fn reserve(&mut self, len: usize, size: usize);

    /// Add the value to the end of the arguments.
    fn add<T>(&mut self, value: T)
    where
        T: Type<Self::Database>,
        T: Encode<Self::Database>;
}
