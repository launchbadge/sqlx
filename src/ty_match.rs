use std::marker::PhantomData;

// These types allow the `query!()` and friends to compare a given parameter's type to
// an expected parameter type even if the former is behind a reference or in `Option`.

// For query parameters, database gives us a single type ID which we convert to an "expected" or
// preferred Rust type, but there can actually be several types that are compatible for a given type
// in input position. E.g. for an expected parameter of `String`, we want to accept `String`,
// `Option<String>`, `&str` and `Option<&str>`. And for the best compiler errors we don't just
// want an `IsCompatible` trait (at least not without `#[on_unimplemented]` which is unstable
// for the foreseeable future).

// We can do this by using autoref (for method calls, the compiler adds reference ops until
// it finds a matching impl) with impls that technically don't overlap as a hacky form of
// specialization (but this works only if all types are statically known, i.e. we're not in a
// generic context; this should suit 99% of use cases for the macros).

#[macro_export]
#[doc(hidden)]
macro_rules! match_type(
    ($($ty:ty : $expr:expr),*,) => ({
        // this shouldn't actually run
        $(if false {
            use $crate::ty_match::{WrapSameExt as _, MatchBorrowExt as _};

            let mut _expr = $crate::ty_match::conjure_value();

            if false {
                // make sure `_expr` has the right type but in a way that the compiler
                // doesn't consider `$expr` to be moved
                _expr = $expr;
                panic!();
            }

            // if `_expr` is `Option<T>`, get `Option<$ty>`, otherwise `$ty`
            let wrapped_same = $crate::ty_match::WrapSame::<$ty, _>::new(&_expr).wrap_same();
            // if `_expr` is `&str`, convert `String` to `&str`
            let mut _ty_check = $crate::ty_match::MatchBorrow::new(&wrapped_same, &_expr).match_borrow();

            // test that `typeof ty_check == typeof $expr`
            _ty_check = $expr;
            // prevents `$expr` from being considered moved
            panic!();
        })*
    })
);

pub struct WrapSame<T, U>(PhantomData<T>, PhantomData<U>);

impl<T, U> WrapSame<T, U> {
    pub fn new(_arg: &U) -> Self {
        WrapSame(PhantomData, PhantomData)
    }
}

pub trait WrapSameExt: Sized {
    type Wrapped;

    fn wrap_same(self) -> Self::Wrapped {
        panic!("only for type resolution")
    }
}

impl<T, U> WrapSameExt for WrapSame<T, Option<U>> {
    type Wrapped = Option<T>;
}

impl<T, U> WrapSameExt for &'_ WrapSame<T, U> {
    type Wrapped = T;
}

pub struct MatchBorrow<T, U>(PhantomData<T>, PhantomData<U>);

impl<T, U> MatchBorrow<T, U> {
    pub fn new(_t: &T, _u: &U) -> Self {
        MatchBorrow(PhantomData, PhantomData)
    }
}

pub trait MatchBorrowExt: Sized {
    type Matched;

    fn match_borrow(self) -> Self::Matched {
        panic!("only for type resolution")
    }
}

impl<'a> MatchBorrowExt for MatchBorrow<Option<String>, Option<&'a str>> {
    type Matched = Option<&'a str>;
}

impl<'a> MatchBorrowExt for MatchBorrow<Option<Vec<u8>>, Option<&'a [u8]>> {
    type Matched = Option<&'a [u8]>;
}

impl<'a> MatchBorrowExt for MatchBorrow<String, &'a str> {
    type Matched = &'a str;
}

impl<'a> MatchBorrowExt for MatchBorrow<Vec<u8>, &'a [u8]> {
    type Matched = &'a [u8];
}

impl<'a, T: 'a, U: 'a> MatchBorrowExt for MatchBorrow<Option<T>, Option<&'a U>> {
    type Matched = Option<&'a T>;
}

impl<'a, T: 'a, U: 'a> MatchBorrowExt for MatchBorrow<T, &'a U> {
    type Matched = &'a T;
}

impl<T, U> MatchBorrowExt for &'_ MatchBorrow<T, U> {
    type Matched = T;
}

pub fn conjure_value<T>() -> T {
    panic!()
}

#[test]
fn test_match_type() {
    match_type!(
        u64: 5u64,
        u64: &5u64,
        u64: &&5u64,
        u64: Some(5u64),
        u64: Some(&5u64),
        u64: Option::<u64>::None,
    );

    match_type!(
        String: "Hello, world",
        String: "Hello, world",
        String: Some("Hello, world"),
    );

    let s = "Hello, world".to_string();

    match_type!(
        String: &s,
        String: &s[..],
        String: Some(&s),
        String: Some(&s[..]),
        String: Some(s.clone()),
        String: s,
    );
}
