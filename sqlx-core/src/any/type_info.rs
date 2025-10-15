use std::fmt::{self, Display, Formatter};

use crate::type_info::TypeInfo;

use AnyTypeInfoKind::*;

#[derive(Debug, Clone, PartialEq)]
pub struct AnyTypeInfo {
    #[doc(hidden)]
    pub kind: AnyTypeInfoKind,
}

impl AnyTypeInfo {
    pub fn kind(&self) -> AnyTypeInfoKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnyTypeInfoKind {
    Null,
    Bool,
    TinyInt,
    SmallInt,
    Integer,
    BigInt,
    UnsignedTinyInt,
    UnsignedSmallInt,
    UnsignedInteger,
    UnsignedBigInt,
    Real,
    Double,
    Text,
    Blob,
}

impl TypeInfo for AnyTypeInfo {
    fn is_null(&self) -> bool {
        self.kind == Null
    }

    fn name(&self) -> &str {
        use AnyTypeInfoKind::*;

        match self.kind {
            Bool => "BOOLEAN",
            TinyInt => "TINYINT",
            SmallInt => "SMALLINT",
            Integer => "INTEGER",
            BigInt => "BIGINT",
            UnsignedTinyInt => "UNSIGNED TINYINT",
            UnsignedSmallInt => "UNSIGNED SMALLINT",
            UnsignedInteger => "UNSIGNED INTEGER",
            UnsignedBigInt => "UNSIGNED BIGINT",
            Real => "REAL",
            Double => "DOUBLE",
            Text => "TEXT",
            Blob => "BLOB",
            Null => "NULL",
        }
    }
}

impl Display for AnyTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl AnyTypeInfoKind {
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            TinyInt
                | SmallInt
                | Integer
                | BigInt
                | UnsignedTinyInt
                | UnsignedSmallInt
                | UnsignedInteger
                | UnsignedBigInt
        )
    }
}
