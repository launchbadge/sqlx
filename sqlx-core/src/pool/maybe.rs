use crate::database::Database;
use crate::pool::PoolConnection;
use std::ops::{Deref, DerefMut};

pub enum MaybePoolConnection<'c, DB: Database> {
    #[allow(dead_code)]
    Connection(&'c mut DB::Connection),
    PoolConnection(PoolConnection<DB>),
}

impl<DB: Database> Deref for MaybePoolConnection<'_, DB> {
    type Target = DB::Connection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            MaybePoolConnection::Connection(v) => v,
            MaybePoolConnection::PoolConnection(v) => v,
        }
    }
}

impl<DB: Database> DerefMut for MaybePoolConnection<'_, DB> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybePoolConnection::Connection(v) => v,
            MaybePoolConnection::PoolConnection(v) => v,
        }
    }
}

impl<DB: Database> From<PoolConnection<DB>> for MaybePoolConnection<'_, DB> {
    fn from(v: PoolConnection<DB>) -> Self {
        MaybePoolConnection::PoolConnection(v)
    }
}

impl<'c, DB: Database> From<&'c mut DB::Connection> for MaybePoolConnection<'c, DB> {
    fn from(v: &'c mut DB::Connection) -> Self {
        MaybePoolConnection::Connection(v)
    }
}
