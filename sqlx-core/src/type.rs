use crate::database::{HasOutput, HasRawValue};
use crate::{decode, encode, Database, Decode, Encode, RawValue, TypeInfo};

/// Indicates that a SQL type is supported for a database.
pub trait Type<Db: Database> {
    /// Returns the canonical SQL type identifier for this Rust type.
    ///
    /// When binding arguments, this is used to tell the database what is about to be sent; which,
    /// the database then uses to guide query plans. This can be overridden by [`type_id_of`].
    ///
    /// A map of SQL types to Rust types is populated with this and used
    /// to determine the type that is returned from the anonymous struct type from [`query!`].
    ///
    fn type_id() -> Db::TypeId
    where
        Self: Sized;

    /// Determines if this Rust type is compatible with the specified SQL type.
    ///
    /// To be compatible, the Rust type must support encoding _and_ decoding
    /// from the specified SQL type.
    ///
    fn compatible(ty: &Db::TypeInfo) -> bool
    where
        Self: Sized,
    {
        ty.id() == Self::type_id()
    }
}

impl<Db: Database, T: Type<Db>> Type<Db> for &'_ T {
    fn type_id() -> Db::TypeId {
        T::type_id()
    }

    fn compatible(ty: &Db::TypeInfo) -> bool {
        T::compatible(ty)
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait TypeEncode<Db: Database>: Type<Db> + Encode<Db> {}

impl<T: Type<Db> + Encode<Db>, Db: Database> TypeEncode<Db> for T {}

#[allow(clippy::module_name_repetitions)]
pub trait TypeDecode<'r, Db: Database>: Sized + Type<Db> + Decode<'r, Db> {}

impl<'r, T: Type<Db> + Decode<'r, Db>, Db: Database> TypeDecode<'r, Db> for T {}

#[allow(clippy::module_name_repetitions)]
pub trait TypeDecodeOwned<Db: Database>: for<'r> TypeDecode<'r, Db> {}

impl<T, Db: Database> TypeDecodeOwned<Db> for T where T: for<'r> TypeDecode<'r, Db> {}
