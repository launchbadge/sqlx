use crate::types::array_compatible;
use crate::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

/// Text type for case insensitive searching in Postgres.
///
/// See https://www.postgresql.org/docs/current/citext.html
///
/// ### Note: Extension Required
/// The `citext` extension is not enabled by default in Postgres. You will need to do so explicitly:
///
/// ```ignore
/// CREATE EXTENSION IF NOT EXISTS "citext";
/// ```

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PgCitext(String);

impl PgCitext {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl Type<Postgres> for PgCitext {
    fn type_info() -> PgTypeInfo {
        // Since `citext` is enabled by an extension, it does not have a stable OID.
        PgTypeInfo::with_name("citext")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <&str as Type<Postgres>>::compatible(ty)
    }
}

impl Deref for PgCitext {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl From<String> for PgCitext {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl FromStr for PgCitext {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PgCitext(s.parse()?))
    }
}

impl Display for PgCitext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PgHasArrayType for PgCitext {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_citext")
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        array_compatible::<&str>(ty)
    }
}

impl Encode<'_, Postgres> for PgCitext {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        <&str as Encode<Postgres>>::encode(&**self, buf)
    }
}

impl Decode<'_, Postgres> for PgCitext {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(PgCitext(value.as_str()?.to_owned()))
    }
}
