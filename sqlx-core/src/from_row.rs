use crate::error::Error;
use crate::row::Row;

/// A record that can be built from a row returned by the database.
///
/// In order to use [`query_as`](crate::query_as) the output type must implement `FromRow`.
///
/// ## Derivable
///
/// This trait can be derived by SQLx for any struct. The generated implementation
/// will consist of a sequence of calls to [`Row::try_get`] using the name from each
/// struct field.
///
/// ```rust,ignore
/// #[derive(sqlx::FromRow)]
/// struct User {
///     id: i32,
///     name: String,
/// }
/// ```
///
/// ### Field attributes
///
/// Several attributes can be specified to customize how each column in a row is read:
///
/// #### `rename`
///
/// When the name of a field in Rust does not match the name of its corresponding column,
/// you can use the `rename` attribute to specify the name that the field has in the row.
/// For example:
///
/// ```rust,ignore
/// #[derive(sqlx::FromRow)]
/// struct User {
///     id: i32,
///     name: String,
///     #[sqlx(rename = "description")]
///     about_me: String
/// }
/// ```
///
/// Given a query such as:
///
/// ```sql
/// SELECT id, name, description FROM users;
/// ```
///
/// will read the content of the column `description` into the field `about_me`.
///
/// #### `rename_all`
/// By default, field names are expected verbatim (with the exception of the raw identifier prefix `r#`, if present).
/// Placed at the struct level, this attribute changes how the field name is mapped to its SQL column name:
///
/// ```rust,ignore
/// #[derive(sqlx::FromRow)]
/// #[sqlx(rename_all = "camelCase")]
/// struct UserPost {
///     id: i32,
///     // remapped to "userId"
///     user_id: i32,
///     contents: String
/// }
/// ```
///
/// The supported values are `snake_case` (available if you have non-snake-case field names for some
/// reason), `lowercase`, `UPPERCASE`, `camelCase`, `PascalCase`, `SCREAMING_SNAKE_CASE` and `kebab-case`.
/// The styling of each option is intended to be an example of its behavior.
///
/// #### `default`
///
/// When your struct contains a field that is not present in your query,
/// if the field type has an implementation for [`Default`],
/// you can use the `default` attribute to assign the default value to said field.
/// For example:
///
/// ```rust,ignore
/// #[derive(sqlx::FromRow)]
/// struct User {
///     id: i32,
///     name: String,
///     #[sqlx(default)]
///     location: Option<String>
/// }
/// ```
///
/// Given a query such as:
///
/// ```sql
/// SELECT id, name FROM users;
/// ```
///
/// will set the value of the field `location` to the default value of `Option<String>`,
/// which is `None`.
///
/// ### `flatten`
///
/// If you want to handle a field that implements [`FromRow`],
/// you can use the `flatten` attribute to specify that you want
/// it to use [`FromRow`] for parsing rather than the usual method.
/// For example:
///
/// ```rust,ignore
/// #[derive(sqlx::FromRow)]
/// struct Address {
///     country: String,
///     city: String,
///     road: String,
/// }
///
/// #[derive(sqlx::FromRow)]
/// struct User {
///     id: i32,
///     name: String,
///     #[sqlx(flatten)]
///     address: Address,
/// }
/// ```
/// Given a query such as:
///
/// ```sql
/// SELECT id, name, country, city, road FROM users;
/// ```
///
/// This field is compatible with the `default` attribute.
///
/// ## Manual implementation
///
/// You can also implement the [`FromRow`] trait by hand. This can be useful if you
/// have a struct with a field that needs manual decoding:
///
///
/// ```rust,ignore
/// use sqlx::{FromRow, sqlite::SqliteRow, sqlx::Row};
/// struct MyCustomType {
///     custom: String,
/// }
///
/// struct Foo {
///     bar: MyCustomType,
/// }
///
/// impl FromRow<'_, SqliteRow> for Foo {
///     fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
///         Ok(Self {
///             bar: MyCustomType {
///                 custom: row.try_get("custom")?
///             }
///         })
///     }
/// }
/// ```
///
/// #### `try_from`
///
/// When your struct contains a field whose type is not matched with the database type,
/// if the field type has an implementation [`TryFrom`] for the database type,
/// you can use the `try_from` attribute to convert the database type to the field type.
/// For example:
///
/// ```rust,ignore
/// #[derive(sqlx::FromRow)]
/// struct User {
///     id: i32,
///     name: String,
///     #[sqlx(try_from = "i64")]
///     bigIntInMySql: u64
/// }
/// ```
///
/// Given a query such as:
///
/// ```sql
/// SELECT id, name, bigIntInMySql FROM users;
/// ```
///
/// In MySql, `BigInt` type matches `i64`, but you can convert it to `u64` by `try_from`.
///
pub trait FromRow<'r, R: Row>: Sized {
    fn from_row(row: &'r R) -> Result<Self, Error>;
}

// implement FromRow for tuples of types that implement Decode
// up to tuples of 9 values

macro_rules! impl_from_row_for_tuple {
    ($( ($idx:tt) -> $T:ident );+;) => {
        impl<'r, R, $($T,)+> FromRow<'r, R> for ($($T,)+)
        where
            R: Row,
            usize: crate::column::ColumnIndex<R>,
            $($T: crate::decode::Decode<'r, R::Database> + crate::types::Type<R::Database>,)+
        {
            #[inline]
            fn from_row(row: &'r R) -> Result<Self, Error> {
                Ok(($(row.try_get($idx as usize)?,)+))
            }
        }
    };
}

impl_from_row_for_tuple!(
    (0) -> T1;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
    (12) -> T13;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
    (12) -> T13;
    (13) -> T14;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
    (12) -> T13;
    (13) -> T14;
    (14) -> T15;
);

impl_from_row_for_tuple!(
    (0) -> T1;
    (1) -> T2;
    (2) -> T3;
    (3) -> T4;
    (4) -> T5;
    (5) -> T6;
    (6) -> T7;
    (7) -> T8;
    (8) -> T9;
    (9) -> T10;
    (10) -> T11;
    (11) -> T12;
    (12) -> T13;
    (13) -> T14;
    (14) -> T15;
    (15) -> T16;
);
