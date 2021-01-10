use std::marker::PhantomData;

use crate::{Database, DefaultRuntime, Runtime};

/// A connection pool to enable the efficient reuse of a managed pool of SQL connections.
pub struct Pool<Db: Database<Rt>, Rt: Runtime = DefaultRuntime> {
    runtime: PhantomData<Rt>,
    database: PhantomData<Db>,
}

// TODO: impl Acquire for &Pool
// TODO: impl Connect for Pool
// TODO: impl Close for Pool
