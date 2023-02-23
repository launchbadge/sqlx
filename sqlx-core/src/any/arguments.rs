use crate::any::value::AnyValueKind;
use crate::any::Any;
use crate::arguments::Arguments;
use crate::encode::Encode;
use crate::types::Type;

pub struct AnyArguments<'q> {
    #[doc(hidden)]
    pub values: AnyArgumentBuffer<'q>,
}

impl<'q> Arguments<'q> for AnyArguments<'q> {
    type Database = Any;

    fn reserve(&mut self, additional: usize, _size: usize) {
        self.values.0.reserve(additional);
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'q + Send + Encode<'q, Self::Database> + Type<Self::Database>,
    {
        let _ = value.encode(&mut self.values);
    }
}

pub struct AnyArgumentBuffer<'q>(#[doc(hidden)] pub Vec<AnyValueKind<'q>>);

impl<'q> Default for AnyArguments<'q> {
    fn default() -> Self {
        AnyArguments {
            values: AnyArgumentBuffer(vec![]),
        }
    }
}

impl<'q> AnyArguments<'q> {
    #[doc(hidden)]
    pub fn convert_to<'a, A: Arguments<'a>>(&'a self) -> A
    where
        'q: 'a,
        Option<i32>: Type<A::Database> + Encode<'a, A::Database>,
        bool: Type<A::Database> + Encode<'a, A::Database>,
        i16: Type<A::Database> + Encode<'a, A::Database>,
        i32: Type<A::Database> + Encode<'a, A::Database>,
        i64: Type<A::Database> + Encode<'a, A::Database>,
        f32: Type<A::Database> + Encode<'a, A::Database>,
        f64: Type<A::Database> + Encode<'a, A::Database>,
        &'a str: Type<A::Database> + Encode<'a, A::Database>,
        &'a [u8]: Type<A::Database> + Encode<'a, A::Database>,
    {
        let mut out = A::default();

        for arg in &self.values.0 {
            match arg {
                AnyValueKind::Null => out.add(Option::<i32>::None),
                AnyValueKind::Bool(b) => out.add(b),
                AnyValueKind::SmallInt(i) => out.add(i),
                AnyValueKind::Integer(i) => out.add(i),
                AnyValueKind::BigInt(i) => out.add(i),
                AnyValueKind::Real(r) => out.add(r),
                AnyValueKind::Double(d) => out.add(d),
                AnyValueKind::Text(t) => out.add(&**t),
                AnyValueKind::Blob(b) => out.add(&**b),
            }
        }

        out
    }
}
