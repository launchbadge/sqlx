use std::fmt;

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

/// SQL Server `XML` column type.
///
/// A newtype wrapper around [`String`] that maps to the MSSQL `XML` type.
/// This allows sqlx macros to distinguish `XML` columns from `NVARCHAR`.
///
/// # Example
///
/// ```rust,no_run
/// # async fn example() -> sqlx::Result<()> {
/// use sqlx::mssql::MssqlXml;
///
/// let xml = MssqlXml::from("<root><item>hello</item></root>".to_owned());
/// assert_eq!(xml.as_ref(), "<root><item>hello</item></root>");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MssqlXml(pub String);

impl Type<Mssql> for MssqlXml {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("XML")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.base_name(),
            "XML" | "NVARCHAR" | "VARCHAR" | "NTEXT" | "TEXT"
        )
    }
}

impl Encode<'_, Mssql> for MssqlXml {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::String(self.0.clone()));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for MssqlXml {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let s = value.as_str()?;
        Ok(MssqlXml(s.to_owned()))
    }
}

impl From<String> for MssqlXml {
    fn from(s: String) -> Self {
        MssqlXml(s)
    }
}

impl From<MssqlXml> for String {
    fn from(xml: MssqlXml) -> Self {
        xml.0
    }
}

impl AsRef<str> for MssqlXml {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MssqlXml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
