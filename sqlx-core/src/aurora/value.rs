use crate::aurora::type_info::{AuroraType, AuroraTypeInfo};
use crate::aurora::Aurora;
use crate::value::{Value, ValueRef};

use rusoto_rds_data::Field;
use std::borrow::Cow;

/// Implementation of [`ValueRef`] for Aurora.
#[derive(Clone)]
pub struct AuroraValueRef<'r> {
    pub(crate) field: &'r Field,
    pub(crate) type_info: AuroraTypeInfo,
}

/// Implementation of [`Value`] for Aurora.
#[derive(Clone)]
pub struct AuroraValue {
    pub(crate) field: Field,
    pub(crate) type_info: AuroraTypeInfo,
}

impl Value for AuroraValue {
    type Database = Aurora;

    #[inline]
    fn as_ref(&self) -> AuroraValueRef<'_> {
        AuroraValueRef {
            field: &self.field,
            type_info: self.type_info,
        }
    }

    fn type_info(&self) -> Cow<'_, AuroraTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        matches!(self.type_info, AuroraTypeInfo(AuroraType::Null))
    }
}

impl<'r> ValueRef<'r> for AuroraValueRef<'r> {
    type Database = Aurora;

    fn to_owned(&self) -> AuroraValue {
        AuroraValue {
            field: self.field.clone(),
            type_info: self.type_info,
        }
    }

    fn type_info(&self) -> Cow<'_, AuroraTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        matches!(self.type_info, AuroraTypeInfo(AuroraType::Null))
    }
}
