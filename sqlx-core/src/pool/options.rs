use std::{marker::PhantomData, time::Duration};

use crate::Backend;

use super::Pool;

#[derive(Default)]
pub struct Builder<DB>
where
    DB: Backend,
{
    phantom: PhantomData<DB>,
    options: Options,
}

impl<DB> Builder<DB>
where
    DB: Backend,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            options: Options::default(),
        }
    }

    pub fn max_size(mut self, max_size: u32) -> Self {
        self.options.max_size = max_size;
        self
    }

    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.options.connect_timeout = connect_timeout;
        self
    }

    pub fn min_idle(mut self, min_idle: u32) -> Self {
        self.options.min_idle = min_idle;
        self
    }

    pub fn max_lifetime(mut self, max_lifetime: impl Into<Option<Duration>>) -> Self {
        self.options.max_lifetime = max_lifetime.into();
        self
    }

    pub fn idle_timeout(mut self, idle_timeout: impl Into<Option<Duration>>) -> Self {
        self.options.idle_timeout = idle_timeout.into();
        self
    }

    pub async fn build(self, url: &str) -> crate::Result<Pool<DB>> {
        Pool::with_options(url, self.options).await
    }
}

pub(crate) struct Options {
    pub max_size: u32,
    pub connect_timeout: Duration,
    pub min_idle: u32,
    pub max_lifetime: Option<Duration>,
    pub idle_timeout: Option<Duration>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: 0,
            connect_timeout: Duration::from_secs(30),
            // 30 minutes
            // prevents unbounded live-leaking of memory due to naive prepared statement caching
            // see src/cache.rs for context
            max_lifetime: Some(Duration::from_secs(1800)),
            idle_timeout: None,
        }
    }
}
