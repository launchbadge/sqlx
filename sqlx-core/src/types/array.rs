use std::ops::{Deref, DerefMut};

/// A generic adapter for encoding and decoding any type that implements
/// [`IntoIterator`][std::iter::IntoIterator]/[`FromIterator`][std::iter::FromIterator]
/// to or from an array in SQL, respectively.
///
/// Only supported on databases that have native support for arrays, such as PostgreSQL.
///
/// ## Examples
///
/// #### (Postgres) Bulk Insert with Array of Structs -> Struct of Arrays
///
/// You can implement bulk insert of structs by turning an array of structs into
/// an array for each field in the struct and then using Postgres' `UNNEST()`
///
/// ```rust,ignore
/// use sqlx::types::Array;
///
/// struct Foo {
///     bar: String,
///     baz: i32,
///     quux: bool
/// }
///
/// let foos = vec![
///     Foo {
///         bar: "bar".to_string(),
///         baz: 0,
///         quux: bool
///     }
/// ];
///
/// sqlx::query!(
///     "
///         INSERT INTO foo(bar, baz, quux)
///         SELECT * FROM UNNEST($1, $2, $3)
///     ",
///      // type overrides are necessary for the macros to accept this instead of `[String]`, etc.
///      Array(foos.iter().map(|foo| &foo.bar)) as _,
///      Array(foos.iter().map(|foo| foo.baz)) as _,
///      Array(foos.iter().map(|foo| foo.quux)) as _
/// )
/// .execute(&pool)
/// .await?;
/// ```
///
/// #### (Postgres) Deserialize a Different Type than `Vec<T>`
///
/// ```sql,ignore
/// CREATE TABLE media(
///     id BIGSERIAL PRIMARY KEY,
///     filename TEXT NOT NULL,
///     tags TEXT[] NOT NULL
/// )
/// ```
///
/// ```rust,ignore
/// use sqlx::types::Array;
///
/// use std::collections::HashSet;
///
/// struct Media {
///     id: i32,
///     filename: String,
///     tags: Array<HashSet<T>>,
/// }
///
/// let media: Vec<Media> = sqlx::query_as!(
///     r#"
///         SELECT id, filename, tags AS "tags: Array<HashSet<_>>"
///     "#
/// )
/// .fetch_all(&pool)
/// .await?;
/// ```
#[derive(Debug)]
pub struct Array<I>(pub I);

impl<I> Array<I> {
    pub fn into_inner(self) -> I {
        self.0
    }
}

impl<I> Deref for Array<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> DerefMut for Array<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I> From<I> for Array<I> {
    fn from(iterable: I) -> Self {
        Self(iterable)
    }
}

// orphan trait impl error
// impl<I> From<Array<I>> for I {
//     fn from(array: Array<I>) -> Self {
//         array.0
//     }
// }
