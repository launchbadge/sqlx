use std::marker::PhantomData;

// These types allow the `query!()` and friends to compare a given parameter's type to
// an expected parameter type even if the former is behind a reference or in `Option`.

// For query parameters, database gives us a single type ID which we convert to an "expected" or
// preferred Rust type, but there can actually be several types that are compatible for a given type
// in input position.

// We can do this by using autoref (for method calls, the compiler adds reference ops until
// it finds a matching impl) with impls that technically don't overlap as a hacky form of
// specialization (but this works only if all types are statically known, i.e. we're not in a
// generic context; this should suit 99% of use cases for the macros).

#[doc(hidden)]
pub struct TyCons<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> TyCons<T> {
    pub fn new(_t: &T) -> TyCons<T> {
        TyCons(PhantomData)
    }
}

#[doc(hidden)]
pub trait TyConsExt: Sized {
    type Cons;
    fn ty_cons(self) -> Self::Cons {
        panic!("should not be run, only for type resolution")
    }

    // if we're two resolutions deep, e.g. trying to match `Option<&str>` with `String`,
    // lift through one of them.
    // https://github.com/launchbadge/sqlx/issues/93
    fn lift(self) -> TyCons<Self::Cons> {
        TyCons::new(&self.ty_cons())
    }
}

impl<T> TyCons<Option<&'_ T>> {
    pub fn ty_cons(self) -> T {
        panic!("should not be run, only for type resolution")
    }
}

// no overlap with the following impls because of the `: Sized` bound
impl<T: Sized> TyConsExt for TyCons<&'_ T> {
    type Cons = T;
}

impl TyConsExt for TyCons<str> {
    type Cons = String;
}

impl<T> TyConsExt for TyCons<[T]> {
    type Cons = Vec<T>;
}

impl TyConsExt for TyCons<&'_ str> {
    type Cons = String;
}

impl<T> TyConsExt for TyCons<&'_ [T]> {
    type Cons = Vec<T>;
}

impl<T> TyConsExt for TyCons<Option<T>> {
    type Cons = T;
}

impl<T> TyConsExt for &'_ TyCons<T> {
    type Cons = T;
}

#[test]
fn test_tycons_ext() {
    if false {
        let _: u64 = TyCons::new(&Some(5u64)).lift().ty_cons();
        let _: u64 = TyCons::new(&Some(&5u64)).lift().ty_cons();
        let _: u64 = TyCons::new(&&5u64).lift().ty_cons();
        let _: u64 = TyCons::new(&5u64).lift().ty_cons();

        // Option<&str>
        let _: String = TyCons::new(&Some("Hello, world!")).lift().ty_cons();
        // Option<String>
        let _: String = TyCons::new(&Some("Hello, world!".to_string()))
            .lift()
            .ty_cons();
        // Option<&String>
        let _: String = TyCons::new(&Some(&"Hello, world!".to_string()))
            .lift()
            .ty_cons();
        // &str
        let _: String = TyCons::new(&"Hello, world!").lift().ty_cons();
        // String
        let _: String = TyCons::new(&"Hello, world!".to_string()).lift().ty_cons();
        // str
        let _: String = TyCons::new("Hello, world!").lift().ty_cons();
    }
}
