mod bool;
mod bytes;
mod float;
mod int;
mod str;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "uuid")]
mod uuid;

#[derive(Debug, Copy, Clone)]
#[repr(i16)]
pub enum TypeFormat {
    Text = 0,
    Binary = 1,
}

impl From<i16> for TypeFormat {
    fn from(code: i16) -> TypeFormat {
        match code {
            0 => TypeFormat::Text,
            1 => TypeFormat::Binary,

            _ => unreachable!(),
        }
    }
}

impl crate::types::HasTypeMetadata for super::Postgres {
    type TypeMetadata = PgTypeMetadata;

    type TableId = u32;

    type TypeId = u32;
}

/// Provides the OIDs for a SQL type and the expected format to be used for
/// transmission between Rust and Postgres.
///
/// While the BINARY format is preferred in most cases, there are scenarios
/// where only the TEXT format may be available for a type.
pub struct PgTypeMetadata {
    #[allow(unused)]
    pub(crate) format: TypeFormat,
    pub(crate) oid: u32,
    pub(crate) array_oid: u32,
}

impl PgTypeMetadata {
    const fn binary(oid: u32, array_oid: u32) -> Self {
        Self {
            format: TypeFormat::Binary,
            oid,
            array_oid,
        }
    }
}

impl PartialEq<u32> for PgTypeMetadata {
    fn eq(&self, other: &u32) -> bool {
        self.oid == *other || self.array_oid == *other
    }
}
