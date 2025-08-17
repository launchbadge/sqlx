use crate::database::Database;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use std::borrow::Cow;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

use crate::types::Type;

#[cfg(feature = "uuid")]
use uuid::Uuid;

pub trait IntoEncode<DB>
where
    DB: Database,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's;

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static;
}

impl<DB, T> IntoEncode<DB> for &T
where
    DB: Database,
    T: for<'e> Encode<'e, DB>,
    T: Clone + IntoEncode<DB> + Debug + Send + Sync,
    T: Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        (*self).clone().into_encode_owned()
    }
}

impl<'a, DB, T> IntoEncode<DB> for &'a [T]
where
    DB: Database,
    Self: for<'e> Encode<'e, DB>,
    Self: ToOwned,
    Self: Type<DB>,
    <&'a [T] as ToOwned>::Owned: Debug + Send + Sync + 'static + Encode<'static, DB> + Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self.to_owned())
    }
}

impl<DB, T, const N: usize> IntoEncode<DB> for [T; N]
where
    DB: Database,
    for<'e> Self: Encode<'e, DB>,
    Self: Type<DB>,
    T: Clone,
    Vec<T>: Debug + Send + Sync + 'static + Encode<'static, DB> + Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's + Type<DB>
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self.to_vec())
    }
}

impl<'a, DB> IntoEncode<DB> for &'a str
where
    Self: 'a,
    DB: Database,
    for<'e> &'e str: Encode<'e, DB> + Type<DB>,
    String: Encode<'static, DB>,
    String: Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self.to_string())
    }
}

pub trait EncodeOwned<DB: Database>: Encode<'static, DB> + Debug + Send + Sync {
    fn type_info(&self) -> DB::TypeInfo;
    fn type_compatible(&self, ty: &DB::TypeInfo) -> bool;
}

#[derive(Debug)]
pub struct EncodeClone<DB: Database, T: Debug + Send + Sync + Type<DB>> {
    value: T,
    db: std::marker::PhantomData<DB>,
}

impl<DB: Database, T: Debug + Send + Sync + Encode<'static, DB> + Type<DB>> Encode<'static, DB>
    for EncodeClone<DB, T>
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        self.value.encode_by_ref(buf)
    }
}

impl<DB: Database, T: Debug + Send + Sync + Encode<'static, DB> + Type<DB>> Type<DB>
    for EncodeClone<DB, T>
{
    fn type_info() -> <DB as Database>::TypeInfo
    where
        Self: Sized,
    {
        T::type_info()
    }
}

impl<DB: Database, T: Debug + Send + Sync + Encode<'static, DB> + Type<DB>> EncodeOwned<DB>
    for EncodeClone<DB, T>
{
    fn type_info(&self) -> DB::TypeInfo {
        <Self as Type<DB>>::type_info()
    }

    fn type_compatible(&self, ty: &DB::TypeInfo) -> bool {
        <Self as Type<DB>>::compatible(ty)
    }
}

impl<DB: Database, T: Debug + Send + Sync + Encode<'static, DB> + Type<DB>> From<T>
    for EncodeClone<DB, T>
{
    fn from(value: T) -> Self {
        Self {
            value,
            db: std::marker::PhantomData,
        }
    }
}

#[macro_export]
macro_rules! impl_into_encode {
    ($t:ty) => {
        impl<DB> $crate::encode_owned::IntoEncode<DB> for $t
        where
            DB: $crate::database::Database,
            Self: for<'e> Encode<'e, DB>,
            Self: Type<DB>,
        {
            fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
            where
                Self: 's,
            {
                self
            }

            fn into_encode_owned(self) -> impl $crate::encode_owned::EncodeOwned<DB> + 'static {
                $crate::encode_owned::EncodeClone::from(self)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_into_encode_for_db {
    ($db:ty, $t:ty) => {
        impl $crate::encode_owned::IntoEncode<$db> for $t
        where
            Self: for<'e> $crate::encode::Encode<'e, $db> + $crate::types::Type<$db>,
        {
            fn into_encode<'s>(
                self,
            ) -> impl $crate::encode::Encode<'s, $db> + $crate::types::Type<$db> + 's
            where
                Self: 's,
            {
                self
            }

            fn into_encode_owned(self) -> impl $crate::encode_owned::EncodeOwned<$db> + 'static {
                $crate::encode_owned::EncodeClone::from(self)
            }
        }
    };
}

impl<DB, T> IntoEncode<DB> for Box<T>
where
    DB: Database,
    Self: for<'e> Encode<'e, DB> + Type<DB>,
    T: Debug + Send + Sync + 'static,
    T: Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self)
    }
}

impl<DB> IntoEncode<DB> for Arc<str>
where
    DB: Database,
    Self: for<'e> Encode<'e, DB> + Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self)
    }
}

impl<DB, T> IntoEncode<DB> for Rc<T>
where
    DB: Database,
    Rc<T>: for<'e> Encode<'e, DB>,
    T: Clone + Debug + Send + Sync + 'static,
    T: for<'e> Encode<'e, DB>,
    T: Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self.as_ref().clone())
    }
}

impl<DB> IntoEncode<DB> for Rc<str>
where
    DB: Database,
    Rc<str>: for<'e> Encode<'e, DB> + Type<DB>,
    String: for<'e> Encode<'e, DB>,
    String: Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self.to_string())
    }
}

impl<DB> IntoEncode<DB> for Rc<[u8]>
where
    DB: Database,
    Rc<[u8]>: for<'e> Encode<'e, DB> + Type<DB>,
    Vec<u8>: for<'e> Encode<'e, DB>,
    Vec<u8>: Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self.as_ref().to_vec())
    }
}

impl<DB, T> IntoEncode<DB> for Arc<T>
where
    DB: Database,
    T: Debug + Send + Sync + 'static,
    Arc<T>: for<'e> Encode<'e, DB>,
    Self: Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self)
    }
}

impl<DB, T> IntoEncode<DB> for Cow<'_, T>
where
    DB: Database,
    T: ToOwned + ?Sized,
    Self: for<'f> Encode<'f, DB> + Type<DB>,
    T::Owned: for<'f> Encode<'f, DB> + Type<DB> + Debug + Send + Sync + 'static,
    T::Owned: IntoEncode<DB> + Clone + Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        <T::Owned as IntoEncode<DB>>::into_encode_owned(self.as_ref().to_owned())
    }
}

impl<DB, T> IntoEncode<DB> for Option<T>
where
    DB: Database,
    T: Encode<'static, DB> + Type<DB> + Debug + Send + Sync,
    Self: for<'e> Encode<'e, DB>,
    Self: Type<DB>,
    // todo: remove after into_encode_owned is fixed
    i32: Encode<'static, DB> + Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        // todo: fix. Needs T to be 'static but that breaks usage in query macros
        EncodeClone::from(0i32)
    }
}

impl<DB, T> IntoEncode<DB> for Vec<T>
where
    DB: Database,
    Vec<T>: for<'e> Encode<'e, DB>,
    T: Clone + Debug + Send + Sync + 'static,
    Self: for<'e> Encode<'e, DB>,
    Self: Type<DB>,
{
    fn into_encode<'s>(self) -> impl Encode<'s, DB> + Type<DB> + 's
    where
        Self: 's,
    {
        self
    }

    fn into_encode_owned(self) -> impl EncodeOwned<DB> + 'static {
        EncodeClone::from(self)
    }
}

impl_into_encode!(bool);

impl_into_encode!(f32);
impl_into_encode!(f64);

impl_into_encode!(i8);
impl_into_encode!(i16);
impl_into_encode!(i32);
impl_into_encode!(i64);

impl_into_encode!(u8);
impl_into_encode!(u16);
impl_into_encode!(u32);
impl_into_encode!(u64);

impl_into_encode!(String);
impl_into_encode!(Box<str>);
impl_into_encode!(Box<[u8]>);
impl_into_encode!(Arc<[u8]>);

#[cfg(feature = "uuid")]
impl_into_encode!(Uuid);
