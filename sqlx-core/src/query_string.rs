use std::sync::Arc;

/// A SQL string that is safe to execute on a database connection.
///
/// A "safe" query string is one that is unlikely to contain a [SQL injection vulnerability][injection].
///
/// In practice, this means a string type that is unlikely to contain dynamic data or user input.
///
/// This is designed to act as a speedbump against naively using `format!()` to add dynamic data
/// or user input to a query, which is a classic vector for SQL injection as SQLx does not
/// provide any sort of escaping or sanitization (which would have to be specially implemented
/// for each database flavor/locale).
///
/// The recommended way to incorporate dynamic data or user input in a query is to use
/// bind parameters, which requires the query to execute as a prepared statement.
/// See [`query()`] for details.
///
/// `&'static str` is the only string type that satisfies the requirements of this trait
/// (ignoring [`String::leak()`] which has niche use-cases) and so is the only string type that
/// natively implements this trait by default.
///
/// For other string types, use [`AssertQuerySafe`] to assert this property.
/// This is the only intended way to pass an owned `String` to [`query()`] and its related functions.
///
/// This trait and `AssertQuerySafe` are intentionally analogous to [`std::panic::UnwindSafe`] and
/// [`std::panic::AssertUnwindSafe`].
///
/// [injection]: https://en.wikipedia.org/wiki/SQL_injection
/// [`query()`]: crate::query::query
pub trait QuerySafeStr<'a> {
    ///
    fn wrap(self) -> QueryString<'a>;
}

impl QuerySafeStr<'static> for &'static str {
    fn wrap(self) -> QueryString<'static> {
        QueryString(Repr::Slice(self))
    }
}

/// Assert that some string type is safe to execute on a database connection.
///
/// Using this API means that **you** have made sure that the string contents do not contain a
/// [SQL injection vulnerability][injection]. It means that, if the string was constructed
/// dynamically, and/or from user input, you have taken care to sanitize the input yourself.
///
/// The maintainers of SQLx take no responsibility for any data leaks or loss resulting from the use
/// of this API. **Use at your own risk.**
///
/// Note that `&'static str` implements [`QuerySafeStr`] directly and so does not need to be wrapped
/// with this type.
///
/// [injection]: https://en.wikipedia.org/wiki/SQL_injection
pub struct AssertQuerySafe<T>(pub T);

impl<'a> QuerySafeStr<'a> for AssertQuerySafe<&'a str> {
    fn wrap(self) -> QueryString<'a> {
        QueryString(Repr::Slice(self.0))
    }
}
impl QuerySafeStr<'static> for AssertQuerySafe<String> {
    fn wrap(self) -> QueryString<'static> {
        // For `Repr` to not be 4 words wide, we convert `String` to `Box<str>`
        QueryString(Repr::Boxed(self.0.into()))
    }
}

impl QuerySafeStr<'static> for AssertQuerySafe<Box<str>> {
    fn wrap(self) -> QueryString<'static> {
        QueryString(Repr::Boxed(self.0))
    }
}

// Note: this is not implemented for `Rc<str>` because it would make `QueryString: !Send`.
impl QuerySafeStr<'static> for AssertQuerySafe<Arc<str>> {
    fn wrap(self) -> QueryString<'static> {
        QueryString(Repr::Arced(self.into()))
    }
}


/// A SQL string that is ready to execute on a database connection.
///
/// This is essentially `Cow<'a, str>` but which can be constructed from additional types
/// without copying.
///
/// See [`QuerySafeStr`] for details.
pub struct QueryString<'a>(Repr<'a>);

impl<'a> QuerySafeStr<'a> for QueryString<'a> {
    fn wrap(self) -> QueryString<'a> {
        self
    }
}

impl QueryString<'_> {
    pub fn into_static(self) -> QueryString<'static> {
        QueryString(match self.0 {
            Repr::Slice(s) => Repr::Boxed(s.into()),
            Repr::StaticSlice(s) => Repr::StaticSlice(s),
            Repr::Boxed(s) => Repr::Boxed(s),
            Repr::Arced(s) => Repr::Arced(s),
        })
    }
}

enum Repr<'a> {
    Slice(&'a str),
    // We need a variant to memoize when we already have a static string, so we don't copy it.
    StaticSlice(&'static str),
    // This enum would be 4 words wide if this variant existed. Instead, convert to `Box<str>`.
    // Owned(String),
    Boxed(Box<str>),
    Arced(Arc<str>),
}
