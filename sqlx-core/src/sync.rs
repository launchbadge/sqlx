// For types with identical signatures that don't require runtime support,
// we can just arbitrarily pick one to use based on what's enabled.
//
// We'll generally lean towards Tokio's types as those are more featureful
// (including `tokio-console` support) and more widely deployed.

#[cfg(feature = "_rt-tokio")]
pub use tokio::sync::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard, RwLock as AsyncRwLock};

#[cfg(all(feature = "_rt-async-lock", not(feature = "_rt-tokio")))]
pub use async_lock::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard, RwLock as AsyncRwLock};

#[cfg(not(any(feature = "_rt-async-lock", feature = "_rt-tokio")))]
pub use noop::*;

#[cfg(not(any(feature = "_rt-async-lock", feature = "_rt-tokio")))]
mod noop {
    use crate::rt::missing_rt;
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};

    pub struct AsyncMutex<T> {
        // `Sync` if `T: Send`
        _marker: PhantomData<std::sync::Mutex<T>>,
    }

    pub struct AsyncMutexGuard<'a, T> {
        inner: &'a AsyncMutex<T>,
    }

    impl<T> AsyncMutex<T> {
        pub fn new(val: T) -> Self {
            missing_rt(val)
        }

        pub fn lock(&self) -> AsyncMutexGuard<T> {
            missing_rt(self)
        }
    }

    impl<T> Deref for AsyncMutexGuard<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            missing_rt(self)
        }
    }

    impl<T> DerefMut for AsyncMutexGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            missing_rt(self)
        }
    }
}
