use crate::{Database, Decode, Encode, TypeInfo};

// NOTE: The interface here is not final. There are some special considerations
//       for MSSQL and Postgres (Arrays and Ranges) that need careful handling
//       to ensure we correctly cover them.

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

#[allow(clippy::module_name_repetitions)]
pub trait TypeEncode<Db: Database>: Type<Db> + Encode<Db> {
    /// Returns the canonical SQL type identifier for this Rust type.
    #[allow(unused_variables)]
    fn type_id(&self, ty: &Db::TypeInfo) -> Db::TypeId;

    /// Determines if this Rust type is compatible with the specified SQL type.
    ///
    /// To be compatible, the Rust type must support encoding _and_ decoding
    /// from the specified SQL type.
    ///
    fn compatible(&self, ty: &Db::TypeInfo) -> bool {
        ty.id() == self.type_id(ty)
    }

    /// Returns the Rust type name of this.
    #[doc(hidden)]
    #[inline]
    fn __rust_type_name_of(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait TypeDecode<'r, Db: Database>: Type<Db> + Decode<'r, Db> {}

impl<'r, T: Type<Db> + Decode<'r, Db>, Db: Database> TypeDecode<'r, Db> for T {}
