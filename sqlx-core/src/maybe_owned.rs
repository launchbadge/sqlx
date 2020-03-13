use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};

pub(crate) enum MaybeOwned<'a, O, B = O> {
    #[allow(dead_code)]
    Borrowed(&'a mut B),

    #[allow(dead_code)]
    Owned(O),
}

impl<'a, O, B> From<&'a mut B> for MaybeOwned<'a, O, B> {
    fn from(val: &'a mut B) -> Self {
        MaybeOwned::Borrowed(val)
    }
}

impl<'a, O, B> Deref for MaybeOwned<'a, O, B>
where
    O: Borrow<B>,
{
    type Target = B;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwned::Borrowed(val) => val,
            MaybeOwned::Owned(ref val) => val.borrow(),
        }
    }
}

impl<'a, O, B> DerefMut for MaybeOwned<'a, O, B>
where
    O: BorrowMut<B>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeOwned::Borrowed(val) => val,
            MaybeOwned::Owned(ref mut val) => val.borrow_mut(),
        }
    }
}
