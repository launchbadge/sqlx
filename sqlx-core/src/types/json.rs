use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
pub use serde_json::value::RawValue as JsonRawValue;
pub use serde_json::Value as JsonValue;

use crate::database::{Database, HasArguments, HasValueRef};
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;

/// Json for json and jsonb fields
///
/// Will attempt to cast to type passed in as the generic.
///
/// ```toml
/// [dependencies]
/// serde_json = { version = "1.0", features = ["raw_value"] }
///
/// ```
///
/// # Example
///
/// ```
/// # use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct Book {
///   name: String
/// }
///
/// #[derive(sqlx::FromRow)]
/// struct Author {
///   name: String,
///   books: sqlx::types::Json<Book>
/// }
/// ```
///
/// Can also be used to turn the json/jsonb into a hashmap
/// ```
/// use std::collections::HashMap;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Book {
///   name: String
/// }
/// #[derive(sqlx::FromRow)]
/// struct Library {
///   id: String,
///   dewey_decimal: sqlx::types::Json<HashMap<String, Book>>
/// }
/// ```
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Json<T: ?Sized>(pub T);

impl<T> From<T> for Json<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<T> for Json<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Json<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

const JSON_SERIALIZE_ERR: &str = "failed to encode value as JSON; the most likely cause is \
                                  attempting to serialize a map with a non-string key type";

// UNSTABLE: for driver use only!
#[doc(hidden)]
impl<T: Serialize> Json<T> {
    pub fn encode_to_string(&self) -> String {
        // Encoding is supposed to be infallible so we don't have much choice but to panic here.
        // However, I believe that's the right thing to do anyway as an object being unable
        // to serialize to JSON is likely due to a bug or a malformed datastructure.
        serde_json::to_string(self).expect(JSON_SERIALIZE_ERR)
    }

    pub fn encode_to(&self, buf: &mut Vec<u8>) {
        serde_json::to_writer(buf, self).expect(JSON_SERIALIZE_ERR)
    }
}

// UNSTABLE: for driver use only!
#[doc(hidden)]
impl<'a, T: 'a> Json<T>
where
    T: Deserialize<'a>,
{
    pub fn decode_from_string(s: &'a str) -> Result<Self, BoxDynError> {
        serde_json::from_str(s).map_err(Into::into)
    }

    pub fn decode_from_bytes(bytes: &'a [u8]) -> Result<Self, BoxDynError> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }
}

impl<DB> Type<DB> for JsonValue
where
    Json<Self>: Type<DB>,
    DB: Database,
{
    fn type_info() -> DB::TypeInfo {
        <Json<Self> as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <Json<Self> as Type<DB>>::compatible(ty)
    }
}

impl<'q, DB> Encode<'q, DB> for JsonValue
where
    for<'a> Json<&'a Self>: Encode<'q, DB>,
    DB: Database,
{
    fn encode_by_ref(&self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <Json<&Self> as Encode<'q, DB>>::encode(Json(self), buf)
    }
}

impl<'r, DB> Decode<'r, DB> for JsonValue
where
    Json<Self>: Decode<'r, DB>,
    DB: Database,
{
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        <Json<Self> as Decode<DB>>::decode(value).map(|item| item.0)
    }
}

impl<DB> Type<DB> for JsonRawValue
where
    for<'a> Json<&'a Self>: Type<DB>,
    DB: Database,
{
    fn type_info() -> DB::TypeInfo {
        <Json<&Self> as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <Json<&Self> as Type<DB>>::compatible(ty)
    }
}

// We don't have to implement Encode for JsonRawValue because that's covered by the default
// implementation for Encode
impl<'r, DB> Decode<'r, DB> for &'r JsonRawValue
where
    Json<Self>: Decode<'r, DB>,
    DB: Database,
{
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        <Json<Self> as Decode<DB>>::decode(value).map(|item| item.0)
    }
}
