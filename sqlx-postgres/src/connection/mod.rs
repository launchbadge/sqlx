use crate::codec::backend::TransactionStatus;
use crate::{PgConnectOptions, Postgres};
use futures_core::future::BoxFuture;
use sqlx_core::connection::Connection;
use sqlx_core::error::Error;
use sqlx_core::io::BufStream;
use sqlx_rt::TcpStream;

mod connect;
mod io;

/// A connection to a PostgreSQL database.
pub struct PgConnection {
    // underlying TCP or UDS stream,
    // wrapped in a potentially TLS stream,
    // wrapped in a buffered stream
    stream: BufStream<TcpStream>,

    // process id of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    process_id: u32,

    // secret key of this backend
    // used to send cancel requests
    #[allow(dead_code)]
    secret_key: u32,

    // status of the connection
    // are we in a transaction?
    transaction_status: TransactionStatus,
}

impl PgConnection {
    pub(crate) const fn new(stream: BufStream<TcpStream>) -> Self {
        Self {
            stream,
            process_id: 0,
            secret_key: 0,
            transaction_status: TransactionStatus::Idle,
        }
    }
}

impl Connection for PgConnection {
    type Database = Postgres;

    type Options = PgConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        unimplemented!()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        unimplemented!()
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        unimplemented!()
    }
}
