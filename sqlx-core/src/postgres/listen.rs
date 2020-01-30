use std::ops::DerefMut;

use futures_core::future::BoxFuture;
use futures_core::stream::Stream;

use crate::connection::Connection;
use crate::executor::Executor;
use crate::pool::PoolConnection;
use crate::postgres::protocol::{Message, NotificationResponse};
use crate::postgres::{PgConnection, PgPool};
use crate::Result;

type PgPoolConnection = PoolConnection<PgConnection>;

/// Extension methods for Postgres connections.
pub trait PgConnectionExt<C: Connection> {
    fn listen(self, channels: &[&str]) -> PgListener<C>;
}

impl PgConnectionExt<PgConnection> for PgConnection {
    /// Register this connection as a listener on the specified channels.
    fn listen(self, channels: &[&str]) -> PgListener<Self> {
        PgListener::new(Some(self), channels, None)
    }
}

impl PgConnectionExt<PgPoolConnection> for PgPoolConnection {
    /// Register this connection as a listener on the specified channels.
    fn listen(self, channels: &[&str]) -> PgListener<Self> {
        PgListener::new(Some(self), channels, None)
    }
}

/// Extension methods for Postgres connection pools.
pub trait PgPoolExt {
    fn listen(&self, channels: &[&str]) -> PgListener<PgPoolConnection>;
}

impl PgPoolExt for PgPool {
    /// Fetch a new connection from the pool and register it as a listener on the specified channel.
    ///
    /// If the underlying connection ever dies, a new connection will be acquired from the pool,
    /// and listening will resume as normal.
    fn listen(&self, channels: &[&str]) -> PgListener<PgPoolConnection> {
        PgListener::new(None, channels, Some(self.clone()))
    }
}

/// A stream of async database notifications.
///
/// Notifications will always correspond to the channel(s) specified this object is created.
pub struct PgListener<C> {
    needs_to_send_listen_cmd: bool,
    connection: Option<C>,
    channels: Vec<String>,
    pool: Option<PgPool>,
}

impl<C> PgListener<C> {
    /// Construct a new instance.
    pub(self) fn new(connection: Option<C>, channels: &[&str], pool: Option<PgPool>) -> Self {
        let channels = channels.iter().map(|chan| String::from(*chan)).collect();
        Self {
            needs_to_send_listen_cmd: true,
            connection,
            channels,
            pool,
        }
    }
}

impl PgListener<PgPoolConnection> {
    /// Receives the next notification available from any of the subscribed channels.
    ///
    /// When a `PgListener` is created from `PgPool.listen(..)`, the `PgListener` will perform
    /// automatic reconnects to the database using the original `PgPool` and will submit a
    /// `LISTEN` command to the database using the same originally specified channels. As such,
    /// this routine will never return `None` when called on a `PgListener` created from a `PgPool`.
    ///
    /// However, if a `PgListener` instance is created outside of the context of a `PgPool`, then
    /// this routine will return `None` when the underlying connection dies. At that point, any
    /// further calls to this routine will also return `None`.
    pub async fn recv(&mut self) -> Option<Result<PgNotification>> {
        loop {
            // Ensure we have an active connection to work with.
            let conn = match &mut self.connection {
                Some(conn) => conn,
                None => match self.get_new_connection().await {
                    // A new connection has been established, bind it and loop.
                    Ok(Some(conn)) => {
                        self.connection = Some(conn);
                        continue;
                    }
                    // No pool is present on this listener, return None.
                    Ok(None) => return None,
                    // We have a pool to work with, but some error has come up. Return the error.
                    // The next call to `recv` will build a new connection if available.
                    Err(err) => return Some(Err(err)),
                },
            };
            // Ensure the current connection has properly registered all listener channels.
            if self.needs_to_send_listen_cmd {
                if let Err(err) = send_listen_query(conn, &self.channels).await {
                    // If we've encountered an error here, test the connection, drop it if needed,
                    // and return the error. The next call to recv will build a new connection if possible.
                    if let Err(_) = conn.ping().await {
                        self.close_conn().await;
                    }
                    return Some(Err(err));
                }
                self.needs_to_send_listen_cmd = false;
            }
            // Await a notification from the DB.
            match conn.receive().await {
                // We've received an async notification, return it.
                Ok(Some(Message::NotificationResponse(notification))) => {
                    return Some(Ok(notification.into()))
                }
                // Protocol error, return the error.
                Ok(Some(msg)) => {
                    return Some(Err(protocol_err!(
                        "unexpected message received from database {:?}",
                        msg
                    )
                    .into()))
                }
                // The connection is dead, ensure that it is dropped, update self state, and loop to try again.
                Ok(None) => {
                    self.close_conn().await;
                    self.needs_to_send_listen_cmd = true;
                    continue;
                }
                // An error has come up, return it.
                Err(err) => return Some(Err(err)),
            }
        }
    }

    /// Consume this listener, returning a `Stream` of notifications.
    pub fn stream(mut self) -> impl Stream<Item = Result<PgNotification>> {
        use async_stream::stream;
        stream! {
            loop {
                match self.recv().await {
                    Some(res) => yield res,
                    None => break,
                }
            }
        }
    }

    /// Fetch a new connection from the connection pool, if a connection pool is available.
    ///
    /// Errors here are transient. `Ok(None)` indicates that no pool is available.
    async fn get_new_connection(&mut self) -> Result<Option<PgPoolConnection>> {
        let pool = match &self.pool {
            Some(pool) => pool,
            None => return Ok(None),
        };
        Ok(Some(pool.acquire().await?))
    }

    /// Close and drop the current connection.
    async fn close_conn(&mut self) {
        if let Some(conn) = self.connection.take() {
            let _ = conn.close().await;
        }
    }
}

impl PgListener<PgConnection> {
    /// Receives the next notification available from any of the subscribed channels.
    ///
    /// If the underlying connection ever dies, this routine will return `None`. Any further calls
    /// to this routine will also return `None`. If automatic reconnect behavior is needed, use
    /// `PgPool.listen(..)`, which will automatically establish a new connection from the pool and
    /// resusbcribe to all channels.
    pub async fn recv(&mut self) -> Option<Result<PgNotification>> {
        loop {
            // Ensure we have an active connection to work with.
            let mut conn = match &mut self.connection {
                Some(conn) => conn,
                None => return None, // This will never practically be hit, but let's make Rust happy.
            };
            // Ensure the current connection has properly registered all listener channels.
            if self.needs_to_send_listen_cmd {
                if let Err(err) = send_listen_query(&mut conn, &self.channels).await {
                    // If we've encountered an error here, test the connection. If the connection
                    // is good, we return the error. Else, we return `None` as the connection is dead.
                    if let Err(_) = conn.ping().await {
                        return None;
                    }
                    return Some(Err(err));
                }
                self.needs_to_send_listen_cmd = false;
            }
            // Await a notification from the DB.
            match conn.receive().await {
                // We've received an async notification, return it.
                Ok(Some(Message::NotificationResponse(notification))) => {
                    return Some(Ok(notification.into()))
                }
                // Protocol error, return the error.
                Ok(Some(msg)) => {
                    return Some(Err(protocol_err!(
                        "unexpected message received from database {:?}",
                        msg
                    )
                    .into()))
                }
                // The connection is dead, return None.
                Ok(None) => return None,
                // An error has come up, return it.
                Err(err) => return Some(Err(err)),
            }
        }
    }

    /// Consume this listener, returning a `Stream` of notifications.
    pub fn stream(mut self) -> impl Stream<Item = Result<PgNotification>> {
        use async_stream::stream;
        stream! {
            loop {
                match self.recv().await {
                    Some(res) => yield res,
                    None => break,
                }
            }
        }
    }
}

impl<C> PgListener<C>
where
    C: Connection,
{
    /// Close this listener stream and its underlying connection.
    pub async fn close(self) -> BoxFuture<'static, Result<()>> {
        match self.connection {
            Some(conn) => conn.close(),
            None => Box::pin(futures_util::future::ok(())),
        }
    }
}

/// An asynchronous message sent from the database.
#[derive(Debug)]
#[non_exhaustive]
pub struct PgNotification {
    /// The PID of the database process which sent this notification.
    pub pid: u32,
    /// The channel of the notification, which can be thought of as a topic.
    pub channel: String,
    /// The payload of the notification.
    pub payload: String,
}

impl From<Box<NotificationResponse>> for PgNotification {
    fn from(src: Box<NotificationResponse>) -> Self {
        Self {
            pid: src.pid,
            channel: src.channel_name,
            payload: src.message,
        }
    }
}

/// Build a query which issues a LISTEN command for each given channel.
fn build_listen_all_query(channels: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    channels.into_iter().fold(String::new(), |mut acc, chan| {
        acc.push_str(r#"LISTEN ""#);
        acc.push_str(chan.as_ref());
        acc.push_str(r#"";"#);
        acc
    })
}

/// Send the structure listen query to the database.
async fn send_listen_query<C: DerefMut<Target = PgConnection>>(
    conn: &mut C,
    channels: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<()> {
    let cmd = build_listen_all_query(channels);
    conn.send(cmd.as_str()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_listen_all_query_with_single_channel() {
        let output = build_listen_all_query(&["test"]);
        assert_eq!(output.as_str(), r#"LISTEN "test";"#);
    }

    #[test]
    fn build_listen_all_query_with_multiple_channels() {
        let output = build_listen_all_query(&["channel.0", "channel.1"]);
        assert_eq!(output.as_str(), r#"LISTEN "channel.0";LISTEN "channel.1";"#);
    }
}
