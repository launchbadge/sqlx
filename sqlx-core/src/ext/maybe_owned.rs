use std::ops::{Deref, DerefMut};

pub enum MaybeOwned<'a, T> {
    Borrowed(&'a mut T),
    Owned(T),
}

impl<'a, T> From<T> for MaybeOwned<'a, T> {
    fn from(v: T) -> Self {
        MaybeOwned::Owned(v)
    }
}

impl<'a, T> From<&'a mut T> for MaybeOwned<'a, T> {
    fn from(v: &'a mut T) -> Self {
        MaybeOwned::Borrowed(v)
    }
}

impl<'a, T> Deref for MaybeOwned<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwned::Borrowed(v) => v,
            MaybeOwned::Owned(v) => v,
        }
    }
}

impl<'a, T> DerefMut for MaybeOwned<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeOwned::Borrowed(v) => v,
            MaybeOwned::Owned(v) => v,
        }
    }
}
