use std::borrow::Cow;
use std::marker::PhantomData;

use crate::database::HasValueRef;
use crate::mssql::{MsSql, MsSqlTypeInfo};
use crate::value::{Value, ValueRef};

/// Implementation of [`ValueRef`] for MSSQL.
#[derive(Clone)]
pub struct MsSqlValueRef<'r> {
    phantom: PhantomData<&'r ()>,
}

impl ValueRef<'_> for MsSqlValueRef<'_> {
    type Database = MsSql;

    fn to_owned(&self) -> MsSqlValue {
        unimplemented!()
    }

    fn type_info(&self) -> Option<Cow<'_, MsSqlTypeInfo>> {
        unimplemented!()
    }

    fn is_null(&self) -> bool {
        unimplemented!()
    }
}

/// Implementation of [`Value`] for MSSQL.
#[derive(Clone)]
pub struct MsSqlValue {}

impl Value for MsSqlValue {
    type Database = MsSql;

    fn as_ref(&self) -> <Self::Database as HasValueRef<'_>>::ValueRef {
        unimplemented!()
    }

    fn type_info(&self) -> Option<Cow<'_, MsSqlTypeInfo>> {
        unimplemented!()
    }

    fn is_null(&self) -> bool {
        unimplemented!()
    }
}
