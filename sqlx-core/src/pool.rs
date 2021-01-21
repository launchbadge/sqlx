use std::marker::PhantomData;

use crate::{Database, Runtime};

/// A connection pool to enable the efficient reuse of a managed pool of SQL connections.
pub struct Pool<Db, Rt>
where
    Rt: Runtime,
    Db: Database<Rt>,
{
    runtime: PhantomData<Rt>,
    database: PhantomData<Db>,
}

// TODO: impl Acquire for &Pool
// TODO: impl Connect for Pool
// TODO: impl Close for Pool
