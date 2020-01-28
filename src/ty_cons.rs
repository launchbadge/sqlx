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

pub struct InnerType<T>(PhantomData<T>);

impl<T: Sized> InnerType<T> {
    fn new(_t: T) -> Self { InnerType(PhantomData) }
}

pub trait InnerTypeExt: Sized {
    type Inner: Sized;
    fn inner_type(self) -> Self::Inner {
        panic!("only for type resolution")
    }
}

impl<T> InnerTypeExt for InnerType<Option<T>> {
    type Inner = T;
}

impl<T> InnerTypeExt for &'_ InnerType<T> {
    type Inner = T;
}

pub struct MatchBorrow<T, U>(PhantomData<T>, PhantomData<U>);

impl<T, U> MatchBorrow<T, U> {
    fn new(_u: U) -> Self { MatchBorrow(PhantomData, PhantomData) }
}

pub trait MatchBorrowExt: Sized {
    type Matched;

    fn match_borrow(self) -> Self::Matched {
        panic!("only for type resolution")
    }
}

impl<'a> MatchBorrowExt for MatchBorrow<String, &'a str> {
    type Matched = &'a str;
}

impl<'a> MatchBorrowExt for MatchBorrow<Vec<u8>, &'a [u8]> {
    type Matched = &'a [u8];
}

impl<'a, T: 'a, U: 'a> MatchBorrowExt for MatchBorrow<T, &'a U> {
    type Matched = &'a U;
}

impl<T, U> MatchBorrowExt for &'_ MatchBorrow<T, U> {
    type Matched = T;
}

fn conjure_value<T>() -> T {
    panic!()
}

#[test]
fn test_tycons_ext() {
    if false {
        let mut arg: u64 = InnerType::new(Some(5u64)).inner_type();
        arg = MatchBorrow::<u64, _>::new(arg).match_borrow();

        let mut arg: &u64 = InnerType::new(Some(&5u64)).inner_type();
        arg = MatchBorrow::<u64, _>::new(arg).match_borrow();

        let mut arg: &u64 = InnerType::new(&5u64).inner_type();
        arg = MatchBorrow::<u64, _>::new(arg).match_borrow();

        let mut arg: &u64 = InnerType::new(&&5u64).inner_type();
        arg = MatchBorrow::<u64, _>::new(arg).match_borrow();

        let mut arg: &str = InnerType::new("Hello, world").inner_type();
        arg = MatchBorrow::<String, _>::new(arg).match_borrow();

        let mut arg: &&str = InnerType::new(&"Hello, world").inner_type();
        arg = MatchBorrow::<String, _>::new(arg).match_borrow();

        let mut arg: &str = InnerType::new(Some("Hello, world")).inner_type();
        arg = MatchBorrow::<String, _>::new(arg).match_borrow();

        let s = "Hello, world".to_string();
        let mut arg: &String = InnerType::new(Some(&s)).inner_type();
        arg = MatchBorrow::<String, _>::new(arg).match_borrow();

        let mut arg: String = InnerType::new(Some(s)).inner_type();
        arg = MatchBorrow::<String, _>::new(arg).match_borrow();

        let s = "Hello, world".to_string();
        let mut arg: &String = InnerType::new(&s).inner_type();
        arg = MatchBorrow::<String, _>::new(arg).match_borrow();

        let mut arg: String = InnerType::new(s).inner_type();
        arg = MatchBorrow::<String, _>::new(arg).match_borrow();
    }
}
