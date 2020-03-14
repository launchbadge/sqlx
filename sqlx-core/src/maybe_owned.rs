use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};

pub(crate) enum MaybeOwned<O, B> {
    #[allow(dead_code)]
    Borrowed(B),

    #[allow(dead_code)]
    Owned(O),
}

impl<O> MaybeOwned<O, usize> {
    #[allow(dead_code)]
    pub(crate) fn resolve<'a, 'b: 'a>(&'a mut self, collection: &'b mut Vec<O>) -> &'a mut O {
        match self {
            MaybeOwned::Owned(ref mut val) => val,
            MaybeOwned::Borrowed(index) => &mut collection[*index],
        }
    }
}

impl<'a, O, B> From<&'a mut B> for MaybeOwned<O, &'a mut B> {
    fn from(val: &'a mut B) -> Self {
        MaybeOwned::Borrowed(val)
    }
}

impl<O, B> Deref for MaybeOwned<O, B>
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

impl<O, B> DerefMut for MaybeOwned<O, B>
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
