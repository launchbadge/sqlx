use std::marker::PhantomData;

// These types allow the `sqlx_macros::query_[as]!()` macros to polymorphically compare a
// given parameter's type to an expected parameter type even if the former
// is behind a reference or in `Option`

#[doc(hidden)]
pub struct TyCons<T>(PhantomData<T>);

impl<T> TyCons<T> {
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
}

impl<T> TyCons<Option<&'_ T>> {
    pub fn ty_cons(self) -> T {
        panic!("should not be run, only for type resolution")
    }
}

impl<T> TyConsExt for TyCons<&'_ T> {
    type Cons = T;
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
        let _: u64 = TyCons::new(&Some(5u64)).ty_cons();
        let _: u64 = TyCons::new(&Some(&5u64)).ty_cons();
        let _: u64 = TyCons::new(&&5u64).ty_cons();
        let _: u64 = TyCons::new(&5u64).ty_cons();
    }
}
