use crate::any::{Any, AnyTypeInfo, AnyTypeInfoKind};
use crate::database::Database;
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::{Value, ValueRef};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AnyValueKind {
    Null(AnyTypeInfoKind),
    Bool(bool),
    TinyInt(i8),
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    UnsignedTinyInt(u8),
    UnsignedSmallInt(u16),
    UnsignedInteger(u32),
    UnsignedBigInt(u64),
    Real(f32),
    Double(f64),
    Text(Arc<String>),
    TextSlice(Arc<str>),
    Blob(Arc<Vec<u8>>),
}

impl AnyValueKind {
    fn type_info(&self) -> AnyTypeInfo {
        AnyTypeInfo {
            kind: match self {
                AnyValueKind::Null(_) => AnyTypeInfoKind::Null,
                AnyValueKind::Bool(_) => AnyTypeInfoKind::Bool,
                AnyValueKind::TinyInt(_) => AnyTypeInfoKind::TinyInt,
                AnyValueKind::SmallInt(_) => AnyTypeInfoKind::SmallInt,
                AnyValueKind::Integer(_) => AnyTypeInfoKind::Integer,
                AnyValueKind::BigInt(_) => AnyTypeInfoKind::BigInt,
                AnyValueKind::UnsignedTinyInt(_) => AnyTypeInfoKind::UnsignedTinyInt,
                AnyValueKind::UnsignedSmallInt(_) => AnyTypeInfoKind::UnsignedSmallInt,
                AnyValueKind::UnsignedInteger(_) => AnyTypeInfoKind::UnsignedInteger,
                AnyValueKind::UnsignedBigInt(_) => AnyTypeInfoKind::UnsignedBigInt,
                AnyValueKind::Real(_) => AnyTypeInfoKind::Real,
                AnyValueKind::Double(_) => AnyTypeInfoKind::Double,
                AnyValueKind::Text(_) => AnyTypeInfoKind::Text,
                AnyValueKind::TextSlice(_) => AnyTypeInfoKind::Text,
                AnyValueKind::Blob(_) => AnyTypeInfoKind::Blob,
            },
        }
    }

    pub(in crate::any) fn unexpected<Expected: Type<Any>>(&self) -> Result<Expected, BoxDynError> {
        Err(format!("expected {}, got {:?}", Expected::type_info(), self).into())
    }

    pub(in crate::any) fn try_integer<T>(&self) -> Result<T, BoxDynError>
    where
        T: Type<Any>
            + TryFrom<i8>
            + TryFrom<i16>
            + TryFrom<i32>
            + TryFrom<i64>
            + TryFrom<u8>
            + TryFrom<u16>
            + TryFrom<u32>
            + TryFrom<u64>,
        BoxDynError: From<<T as TryFrom<i8>>::Error>,
        BoxDynError: From<<T as TryFrom<i16>>::Error>,
        BoxDynError: From<<T as TryFrom<i32>>::Error>,
        BoxDynError: From<<T as TryFrom<i64>>::Error>,
        BoxDynError: From<<T as TryFrom<u8>>::Error>,
        BoxDynError: From<<T as TryFrom<u16>>::Error>,
        BoxDynError: From<<T as TryFrom<u32>>::Error>,
        BoxDynError: From<<T as TryFrom<u64>>::Error>,
    {
        Ok(match self {
            AnyValueKind::TinyInt(i) => (*i).try_into()?,
            AnyValueKind::SmallInt(i) => (*i).try_into()?,
            AnyValueKind::Integer(i) => (*i).try_into()?,
            AnyValueKind::BigInt(i) => (*i).try_into()?,
            AnyValueKind::UnsignedTinyInt(i) => (*i).try_into()?,
            AnyValueKind::UnsignedSmallInt(i) => (*i).try_into()?,
            AnyValueKind::UnsignedInteger(i) => (*i).try_into()?,
            AnyValueKind::UnsignedBigInt(i) => (*i).try_into()?,
            _ => return self.unexpected(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct AnyValue {
    #[doc(hidden)]
    pub kind: AnyValueKind,
}

#[derive(Clone, Debug)]
pub struct AnyValueRef<'a> {
    pub(crate) kind: &'a AnyValueKind,
}

impl Value for AnyValue {
    type Database = Any;

    fn as_ref(&self) -> <Self::Database as Database>::ValueRef<'_> {
        AnyValueRef { kind: &self.kind }
    }

    fn type_info(&self) -> Cow<'_, <Self::Database as Database>::TypeInfo> {
        Cow::Owned(self.kind.type_info())
    }

    fn is_null(&self) -> bool {
        matches!(self.kind, AnyValueKind::Null(_))
    }
}

impl<'a> ValueRef<'a> for AnyValueRef<'a> {
    type Database = Any;

    fn to_owned(&self) -> <Self::Database as Database>::Value {
        AnyValue {
            kind: self.kind.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, <Self::Database as Database>::TypeInfo> {
        Cow::Owned(self.kind.type_info())
    }

    fn is_null(&self) -> bool {
        matches!(self.kind, AnyValueKind::Null(_))
    }
}
