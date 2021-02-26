use std::any;

use crate::database::HasRawValue;
use crate::{decode, Database, Decode, TypeDecode, TypeInfo};

// NOTE: Add decode() and decode_unchecked() to RawValue as provided methods
//       once Rust has lazy normalization and/or GATs.
pub trait RawValue<'r>: Sized {
    type Database: Database;

    /// Returns `true` if this value is the SQL `NULL`.
    fn is_null(&self) -> bool;

    /// Returns the type information for this value.
    fn type_info(&self) -> &'r <Self::Database as Database>::TypeInfo;
}
