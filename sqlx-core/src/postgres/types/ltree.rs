use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::{Error, BoxDynError};
use crate::postgres::{PgArgumentBuffer, PgValueFormat, PgTypeInfo, PgValueRef, Postgres};
use crate::types::Type;

#[derive(Debug)]
pub struct PgLTree {
    labels: Vec<String>
}

impl FromStr for PgLTree {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(
            Self {
                labels: s.split('.').map(|s| s.to_owned()).collect()
            }
        )
    }
}

impl Display for PgLTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.labels.join("."))
    }
}

impl Type<Postgres> for PgLTree {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::LTREE
    }
}

impl Encode<'_, Postgres> for PgLTree {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        buf.extend(1i8.to_le_bytes());
        buf.extend(self.to_string().as_bytes());

        IsNull::No
    }
}

impl<'r> Decode<'r, Postgres> for PgLTree {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.format() {
            PgValueFormat::Binary => {
                let bytes = value.as_bytes()?;
                let version = i8::from_le_bytes([bytes[0]; 1]);
                if version != 1 {
                    todo!("add error here")
                }
                Ok(Self::from_str(&String::from_utf8(bytes[1..].to_vec())?)?)
            },
            PgValueFormat::Text => Ok(Self::from_str(value.as_str()?)?)
        }
    }
}
