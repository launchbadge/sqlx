//! Provides [`Decode`] for decoding values from the database.

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use crate::database::Database;
use crate::error::BoxDynError;

use crate::value::ValueRef;

/// A type that can be decoded from the database.
///
/// ## How can I implement `Decode`?
///
/// A manual implementation of `Decode` can be useful when adding support for
/// types externally to SQLx.
///
/// The following showcases how to implement `Decode` to be generic over [`Database`]. The
/// implementation can be marginally simpler if you remove the `DB` type parameter and explicitly
/// use the concrete [`ValueRef`](Database::ValueRef) and [`TypeInfo`](Database::TypeInfo) types.
///
/// ```rust
/// # use sqlx_core::database::{Database};
/// # use sqlx_core::decode::Decode;
/// # use sqlx_core::types::Type;
/// # use std::error::Error;
/// #
/// struct MyType;
///
/// # impl<DB: Database> Type<DB> for MyType {
/// # fn type_info() -> DB::TypeInfo { todo!() }
/// # }
/// #
/// # impl std::str::FromStr for MyType {
/// # type Err = sqlx_core::error::Error;
/// # fn from_str(s: &str) -> Result<Self, Self::Err> { todo!() }
/// # }
/// #
/// // DB is the database driver
/// // `'r` is the lifetime of the `Row` being decoded
/// impl<'r, DB: Database> Decode<'r, DB> for MyType
/// where
///     // we want to delegate some of the work to string decoding so let's make sure strings
///     // are supported by the database
///     &'r str: Decode<'r, DB>
/// {
///     fn decode(
///         value: <DB as Database>::ValueRef<'r>,
///     ) -> Result<MyType, Box<dyn Error + 'static + Send + Sync>> {
///         // the interface of ValueRef is largely unstable at the moment
///         // so this is not directly implementable
///
///         // however, you can delegate to a type that matches the format of the type you want
///         // to decode (such as a UTF-8 string)
///
///         let value = <&str as Decode<DB>>::decode(value)?;
///
///         // now you can parse this into your type (assuming there is a `FromStr`)
///
///         Ok(value.parse()?)
///     }
/// }
/// ```
pub trait Decode<'r, DB: Database>: Sized {
    /// Decode a new value of this type using a raw value from the database.
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError>;
}

// implement `Decode` for Option<T> for all SQL types
impl<'r, DB, T> Decode<'r, DB> for Option<T>
where
    DB: Database,
    T: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(T::decode(value)?))
        }
    }
}

macro_rules! impl_decode_for_smartpointer {
    ($smart_pointer:tt) => {
        impl<'r, DB, T> Decode<'r, DB> for $smart_pointer<T>
        where
            DB: Database,
            T: Decode<'r, DB>,
        {
            fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
                Ok(Self::new(T::decode(value)?))
            }
        }

        impl<'r, DB> Decode<'r, DB> for $smart_pointer<str>
        where
            DB: Database,
            &'r str: Decode<'r, DB>,
        {
            fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
                let ref_str = <&str as Decode<DB>>::decode(value)?;
                Ok(ref_str.into())
            }
        }

        impl<'r, DB> Decode<'r, DB> for $smart_pointer<[u8]>
        where
            DB: Database,
            Vec<u8>: Decode<'r, DB>,
        {
            fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
                // The `Postgres` implementation requires this to be decoded as an owned value because
                // bytes can be sent in text format.
                let bytes = <Vec<u8> as Decode<DB>>::decode(value)?;
                Ok(bytes.into())
            }
        }
    };
}

impl_decode_for_smartpointer!(Arc);
impl_decode_for_smartpointer!(Box);
impl_decode_for_smartpointer!(Rc);

// implement `Decode` for Cow<T> for all SQL types
impl<'r, DB, T> Decode<'r, DB> for Cow<'_, T>
where
    DB: Database,
    // `ToOwned` is required here to satisfy `Cow`
    T: ToOwned + ?Sized,
    <T as ToOwned>::Owned: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        // See https://github.com/launchbadge/sqlx/pull/3674#discussion_r2008611502 for more info
        // about why decoding to a `Cow::Owned` was chosen.
        <<T as ToOwned>::Owned as Decode<DB>>::decode(value).map(Cow::Owned)
    }
}
