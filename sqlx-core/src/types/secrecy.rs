/// Conversions between `secrecy::Secret<T>` and SQL types.
use crate::database::{Database, HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;

pub use secrecy::{ExposeSecret, Secret, Zeroize};

impl<DB, T> Type<DB> for Secret<T>
where
    DB: Database,
    T: Type<DB> + Zeroize,
{
    fn type_info() -> DB::TypeInfo {
        T::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        T::compatible(ty)
    }
}

impl<'r, DB, T> Decode<'r, DB> for Secret<T>
where
    DB: Database,
    T: Decode<'r, DB> + Zeroize,
{
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        T::decode(value).map(Secret::new)
    }
}

impl<'q, DB, T> Encode<'q, DB> for Secret<T>
where
    DB: Database,
    T: Encode<'q, DB> + Zeroize,
{
    #[inline]
    fn encode(self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        self.expose_secret().encode(buf)
    }

    #[inline]
    fn encode_by_ref(&self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        self.expose_secret().encode_by_ref(buf)
    }

    #[inline]
    fn produces(&self) -> Option<DB::TypeInfo> {
        self.expose_secret().produces()
    }

    #[inline]
    fn size_hint(&self) -> usize {
        self.expose_secret().size_hint()
    }
}
