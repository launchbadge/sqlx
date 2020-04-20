use std::ops::{Deref, DerefMut};

use crate::sqlite::statement::SqliteStatement;

pub(crate) enum MaybeOwnedStatement<'c> {
    Borrowed(&'c mut SqliteStatement),
    Owned(SqliteStatement),
}

impl Deref for MaybeOwnedStatement<'_> {
    type Target = SqliteStatement;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwnedStatement::Borrowed(v) => v,
            MaybeOwnedStatement::Owned(v) => v,
        }
    }
}

impl DerefMut for MaybeOwnedStatement<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeOwnedStatement::Borrowed(v) => v,
            MaybeOwnedStatement::Owned(v) => v,
        }
    }
}
