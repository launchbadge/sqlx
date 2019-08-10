use crate::{
    backend::Backend,
    connection::{Connection, ConnectionAssocQuery},
    pool::{Pool, PoolOptions},
};
use std::{io, ops::DerefMut};

pub struct Client<DB: Backend> {
    pool: Pool<DB::Connection>,
}

impl<DB: Backend> Clone for Client<DB> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl<DB: Backend> Client<DB> {
    pub fn new(url: &str) -> Self {
        Self {
            pool: Pool::new(
                url,
                PoolOptions {
                    idle_timeout: None,
                    connection_timeout: None,
                    max_lifetime: None,
                    max_size: 70,
                    min_idle: None,
                },
            ),
        }
    }

    pub async fn get(&self) -> io::Result<impl DerefMut<Target = DB::Connection>> {
        Ok(self.pool.acquire().await?)
    }
}

// impl<'c, 'q, DB: Backend> ConnectionAssocQuery<'c, 'q> for Client<DB> {
//     type Query = <<DB as Backend>::Connection as ConnectionAssocQuery<'c, 'q>>::Query;
// }

// impl<DB: Backend> Connection for Client<DB> {
//     type Backend = DB;

//     #[inline]
//     fn establish(url: &str) -> BoxFuture<io::Result<Self>> {
//         Box::pin(future::ok(Client::new(url)))
//     }

//     #[inline]
//     fn prepare<'c, 'q>(&'c mut self, query: &'q str) -> <<DB as Backend>::Connection as ConnectionAssocQuery<'c, 'q>>::Query {
//         // TODO: Think on how to handle error here
//         self.pool.acquire().unwrap().prepare(query)
//     }
// }
