use crate::database::Database;

#[cfg(not(feature = "_rt-wasm-bindgen"))]
use crate::pool::PoolConnection;
use std::ops::{Deref, DerefMut};

pub(crate) enum MaybePoolConnection<'c, DB: Database> {
    #[allow(dead_code)]
    Connection(&'c mut DB::Connection),
    #[cfg(not(feature = "_rt-wasm-bindgen"))]
    PoolConnection(PoolConnection<DB>),
}

impl<'c, DB: Database> Deref for MaybePoolConnection<'c, DB> {
    type Target = DB::Connection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            MaybePoolConnection::Connection(v) => v,
            #[cfg(not(feature = "_rt-wasm-bindgen"))]
            MaybePoolConnection::PoolConnection(v) => v,
        }
    }
}

impl<'c, DB: Database> DerefMut for MaybePoolConnection<'c, DB> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybePoolConnection::Connection(v) => v,
            #[cfg(not(feature = "_rt-wasm-bindgen"))]
            MaybePoolConnection::PoolConnection(v) => v,
        }
    }
}

#[allow(unused_macros)]
macro_rules! impl_into_maybe_pool {
    ($DB:ident, $C:ident) => {
        #[cfg(not(feature = "_rt-wasm-bindgen"))]
        impl<'c> From<crate::pool::PoolConnection<$DB>>
            for crate::pool::MaybePoolConnection<'c, $DB>
        {
            fn from(v: crate::pool::PoolConnection<$DB>) -> Self {
                crate::pool::MaybePoolConnection::PoolConnection(v)
            }
        }

        impl<'c> From<&'c mut $C> for crate::pool::MaybePoolConnection<'c, $DB> {
            fn from(v: &'c mut $C) -> Self {
                crate::pool::MaybePoolConnection::Connection(v)
            }
        }
    };
}
