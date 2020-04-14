use crate::postgres::protocol::TypeId;
use crate::types::TypeInfo;
use std::borrow::Borrow;
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

/// Type information for a Postgres SQL type.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct PgTypeInfo {
    pub(crate) id: Option<TypeId>,
    pub(crate) name: SharedStr,
}

impl PgTypeInfo {
    pub(crate) fn new(id: TypeId, name: impl Into<SharedStr>) -> Self {
        Self {
            id: Some(id),
            name: name.into(),
        }
    }

    /// Create a `PgTypeInfo` from a type name.
    ///
    /// The OID for the type will be fetched from Postgres on bind or decode of
    /// a value of this type. The fetched OID will be cached per-connection.
    pub const fn with_name(name: &'static str) -> Self {
        Self {
            id: None,
            name: SharedStr::Static(name),
        }
    }

    #[doc(hidden)]
    pub fn type_feature_gate(&self) -> Option<&'static str> {
        match self.id? {
            TypeId::DATE | TypeId::TIME | TypeId::TIMESTAMP | TypeId::TIMESTAMPTZ => Some("chrono"),
            TypeId::UUID => Some("uuid"),
            TypeId::JSON | TypeId::JSONB => Some("json"),
            // we can support decoding `PgNumeric` but it's decidedly less useful to the layman
            TypeId::NUMERIC => Some("bigdecimal"),
            TypeId::CIDR | TypeId::INET => Some("ipnetwork"),

            _ => None,
        }
    }
}

impl Display for PgTypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq<PgTypeInfo> for PgTypeInfo {
    fn eq(&self, other: &PgTypeInfo) -> bool {
        // Postgres is strongly typed (mostly) so the rules that make sense here are equivalent
        // to the rules that make sense in [compatible]
        self.compatible(other)
    }
}

impl TypeInfo for PgTypeInfo {
    fn compatible(&self, other: &Self) -> bool {
        if let (Some(self_id), Some(other_id)) = (self.id, other.id) {
            return match (self_id, other_id) {
                (TypeId::CIDR, TypeId::INET)
                | (TypeId::INET, TypeId::CIDR)
                | (TypeId::ARRAY_CIDR, TypeId::ARRAY_INET)
                | (TypeId::ARRAY_INET, TypeId::ARRAY_CIDR) => true,

                // the following text-like types are compatible
                (TypeId::VARCHAR, other)
                | (TypeId::TEXT, other)
                | (TypeId::BPCHAR, other)
                | (TypeId::NAME, other)
                | (TypeId::UNKNOWN, other)
                    if match other {
                        TypeId::VARCHAR
                        | TypeId::TEXT
                        | TypeId::BPCHAR
                        | TypeId::NAME
                        | TypeId::UNKNOWN => true,
                        _ => false,
                    } =>
                {
                    true
                }

                // the following text-like array types are compatible
                (TypeId::ARRAY_VARCHAR, other)
                | (TypeId::ARRAY_TEXT, other)
                | (TypeId::ARRAY_BPCHAR, other)
                | (TypeId::ARRAY_NAME, other)
                    if match other {
                        TypeId::ARRAY_VARCHAR
                        | TypeId::ARRAY_TEXT
                        | TypeId::ARRAY_BPCHAR
                        | TypeId::ARRAY_NAME => true,
                        _ => false,
                    } =>
                {
                    true
                }

                // JSON <=> JSONB
                (TypeId::JSON, other) | (TypeId::JSONB, other)
                    if match other {
                        TypeId::JSON | TypeId::JSONB => true,
                        _ => false,
                    } =>
                {
                    true
                }

                _ => self_id.0 == other_id.0,
            };
        }

        // If the type names match, the types are equivalent (and compatible)
        // If the type names are the empty string, they are invalid type names

        if (&*self.name == &*other.name) && !self.name.is_empty() {
            return true;
        }

        // TODO: More efficient way to do case insensitive comparison
        if !self.name.is_empty() && (&*self.name.to_lowercase() == &*other.name.to_lowercase()) {
            return true;
        }

        false
    }
}

/// Copy of `Cow` but for strings; clones guaranteed to be cheap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SharedStr {
    Static(&'static str),
    Arc(Arc<str>),
}

impl Deref for SharedStr {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            SharedStr::Static(s) => s,
            SharedStr::Arc(s) => s,
        }
    }
}

impl Hash for SharedStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Forward the hash to the string representation of this
        // A derive(Hash) encodes the enum discriminant
        (&**self).hash(state);
    }
}

impl Borrow<str> for SharedStr {
    fn borrow(&self) -> &str {
        &**self
    }
}

impl<'a> From<&'a SharedStr> for SharedStr {
    fn from(s: &'a SharedStr) -> Self {
        s.clone()
    }
}

impl From<&'static str> for SharedStr {
    fn from(s: &'static str) -> Self {
        SharedStr::Static(s)
    }
}

impl From<String> for SharedStr {
    #[inline]
    fn from(s: String) -> Self {
        SharedStr::Arc(s.into())
    }
}

impl From<SharedStr> for String {
    fn from(s: SharedStr) -> Self {
        String::from(&*s)
    }
}

impl fmt::Display for SharedStr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(self)
    }
}

// manual impls because otherwise things get a little screwy with lifetimes
#[cfg(feature = "offline")]
impl<'de> serde::Deserialize<'de> for SharedStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)?.into())
    }
}

#[cfg(feature = "offline")]
impl serde::Serialize for SharedStr {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self)
    }
}
