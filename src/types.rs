use crate::backend::Backend;

/// Information about how a backend stores metadata about
/// given SQL types.
pub trait TypeMetadata {
    /// The actual type used to represent metadata.
    type TypeMetadata;
}

/// Indicates that a SQL type exists for a backend and defines
/// useful metadata for the backend.
pub trait HasSqlType<A>: TypeMetadata {
    fn metadata() -> Self::TypeMetadata;
}

// TODO: #[derive(SqlType)]
// pub struct Text<'a>(Cow<'a, str>);

// TODO: #[derive(SqlType)]
// pub struct SmallInt(i16);

// TODO: #[derive(SqlType)]
// pub struct Int(i32);

// TODO: #[derive(SqlType)]
// pub struct BigInt(i64);

// TODO: #[derive(SqlType)]
// pub struct Real(f32);

// TODO: #[derive(SqlType)]
// pub struct Double(f64);

// Example of what that derive should generate

// impl HasSqlType<Bool> for Pg {
//     #[inline]
//     fn metadata() -> PgTypeMetadata {
//         <Pg as HasSqlType<bool>>::metadata()
//     }
// }

// impl ToSql<Pg> for Bool {
//     #[inline]
//     fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
//         self.0.to_sql(buf)
//     }
// }

// impl FromSql<Pg> for bool {
//     #[inline]
//     fn from_sql(buf: Option<&[u8]>) -> Self {
//         Self(bool::from_sql(buf))
//     }
// }
