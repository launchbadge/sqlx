use crate::any::value::AnyValueKind;
use crate::any::{Any, AnyTypeInfoKind};
use crate::arguments::Arguments;
use crate::encode::{Encode, IsNull};
use crate::encode_owned::IntoEncode;
use crate::error::BoxDynError;
use crate::types::Type;
use std::sync::Arc;

#[derive(Default)]
pub struct AnyArguments {
    #[doc(hidden)]
    pub values: AnyArgumentBuffer,
}

impl Arguments for AnyArguments {
    type Database = Any;

    fn reserve(&mut self, additional: usize, _size: usize) {
        self.values.0.reserve(additional);
    }

    fn add<'t, T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: IntoEncode<Self::Database> + Type<Self::Database>,
    {
        let _: IsNull = value.into_encode().encode(&mut self.values)?;
        Ok(())
    }

    fn len(&self) -> usize {
        self.values.0.len()
    }
}

#[derive(Default)]
pub struct AnyArgumentBuffer(#[doc(hidden)] pub Vec<AnyValueKind>);

impl AnyArguments {
    #[doc(hidden)]
    pub fn convert_into<A: Arguments>(self) -> Result<A, BoxDynError>
    where
        Option<i32>: IntoEncode<A::Database> + Type<A::Database>,
        Option<bool>: IntoEncode<A::Database> + Type<A::Database>,
        Option<i16>: IntoEncode<A::Database> + Type<A::Database>,
        Option<i32>: IntoEncode<A::Database> + Type<A::Database>,
        Option<i64>: IntoEncode<A::Database> + Type<A::Database>,
        Option<f32>: IntoEncode<A::Database> + Type<A::Database>,
        Option<f64>: IntoEncode<A::Database> + Type<A::Database>,
        Option<String>: IntoEncode<A::Database> + Type<A::Database>,
        Option<Vec<u8>>: IntoEncode<A::Database> + Type<A::Database>,
        bool: IntoEncode<A::Database> + Type<A::Database>,
        i16: IntoEncode<A::Database> + Type<A::Database>,
        i32: IntoEncode<A::Database> + Type<A::Database>,
        i64: IntoEncode<A::Database> + Type<A::Database>,
        f32: IntoEncode<A::Database> + Type<A::Database>,
        f64: IntoEncode<A::Database> + Type<A::Database>,
        String: IntoEncode<A::Database> + Type<A::Database>,
        Vec<u8>: IntoEncode<A::Database> + Type<A::Database>,
        Arc<String>: IntoEncode<A::Database> + Type<A::Database>,
        Arc<str>: IntoEncode<A::Database> + Type<A::Database>,
        Arc<Vec<u8>>: IntoEncode<A::Database> + Type<A::Database>,
    {
        let mut out = A::default();

        for arg in self.values.0 {
            match arg {
                AnyValueKind::Null(AnyTypeInfoKind::Null) => out.add(Option::<i32>::None),
                AnyValueKind::Null(AnyTypeInfoKind::Bool) => out.add(Option::<bool>::None),
                AnyValueKind::Null(AnyTypeInfoKind::SmallInt) => out.add(Option::<i16>::None),
                AnyValueKind::Null(AnyTypeInfoKind::Integer) => out.add(Option::<i32>::None),
                AnyValueKind::Null(AnyTypeInfoKind::BigInt) => out.add(Option::<i64>::None),
                AnyValueKind::Null(AnyTypeInfoKind::Real) => out.add(Option::<f64>::None),
                AnyValueKind::Null(AnyTypeInfoKind::Double) => out.add(Option::<f32>::None),
                AnyValueKind::Null(AnyTypeInfoKind::Text) => out.add(Option::<String>::None),
                AnyValueKind::Null(AnyTypeInfoKind::Blob) => out.add(Option::<Vec<u8>>::None),
                AnyValueKind::Bool(b) => out.add(b),
                AnyValueKind::SmallInt(i) => out.add(i),
                AnyValueKind::Integer(i) => out.add(i),
                AnyValueKind::BigInt(i) => out.add(i),
                AnyValueKind::Real(r) => out.add(r),
                AnyValueKind::Double(d) => out.add(d),
                AnyValueKind::Text(t) => out.add(t),
                AnyValueKind::TextSlice(t) => out.add(t),
                AnyValueKind::Blob(b) => out.add(b),
            }?
        }
        Ok(out)
    }
}
