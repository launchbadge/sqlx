use std::ops::DerefMut;

use async_stream::stream;
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
pub trait PgConnectionExt<C: Connection + Unpin> {
    fn listen(self, channels: &[&str]) -> PgListener<C>;
}

impl PgConnectionExt<PgConnection> for PgConnection {
    /// Register this connection as a listener on the specified channels.
    fn listen(self, channels: &[&str]) -> PgListener<Self> {
        PgListener::new(self, channels)
    }
}

impl PgConnectionExt<PgPoolConnection> for PgPoolConnection {
    /// Register this connection as a listener on the specified channels.
    fn listen(self, channels: &[&str]) -> PgListener<Self> {
        PgListener::new(self, channels)
    }
}

/// A stream of async database notifications.
///
/// Notifications will always correspond to the channel(s) specified this object is created.
///
/// This listener is bound to the lifetime of its underlying connection. If the connection ever
/// dies, this listener will terminate and will no longer yield any notifications.
pub struct PgListener<C> {
    needs_to_send_listen_cmd: bool,
    connection: C,
    channels: Vec<String>,
}

impl<C> PgListener<C> {
    /// Construct a new instance.
    pub(self) fn new(connection: C, channels: &[&str]) -> Self {
        let channels = channels.iter().map(|chan| String::from(*chan)).collect();
        Self {
            needs_to_send_listen_cmd: true,
            connection,
            channels,
        }
    }
}

impl<C> PgListener<C>
where
    C: Connection,
    C: DerefMut<Target = PgConnection>,
{
    /// Receives the next notification available from any of the subscribed channels.
    pub async fn recv(&mut self) -> Result<Option<PgNotification>> {
        loop {
            // Ensure the current connection has properly registered all listener channels.
            if self.needs_to_send_listen_cmd {
                if let Err(err) = send_listen_query(&mut self.connection, &self.channels).await {
                    // If we've encountered an error here, test the connection. If the connection
                    // is good, we return the error. Else, we return `None` as the connection is dead.
                    if let Err(_) = self.connection.ping().await {
                        return Ok(None);
                    }
                    return Err(err);
                }
                self.needs_to_send_listen_cmd = false;
            }
            // Await a notification from the DB.
            match self.connection.receive().await? {
                // We've received an async notification, return it.
                Some(Message::NotificationResponse(notification)) => {
                    return Ok(Some(notification.into()))
                }
                // Protocol error, return the error.
                Some(msg) => {
                    return Err(protocol_err!(
                        "unexpected message received from database {:?}",
                        msg
                    )
                    .into())
                }
                // The connection is dead, return None.
                None => return Ok(None),
            }
        }
    }

    /// Consume this listener, returning a `Stream` of notifications.
    pub fn into_stream(mut self) -> impl Stream<Item = Result<Option<PgNotification>>> {
        stream! {
            loop {
                yield self.recv().await
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
        self.connection.close()
    }
}

/// Extension methods for Postgres connection pools.
pub trait PgPoolExt {
    fn listen(&self, channels: &[&str]) -> PgPoolListener;
}

impl PgPoolExt for PgPool {
    /// Create a listener which supports automatic reconnects using the connection pool.
    fn listen(&self, channels: &[&str]) -> PgPoolListener {
        PgPoolListener::new(channels, self.clone())
    }
}

/// A stream of async database notifications.
///
/// Notifications will always correspond to the channel(s) specified this object is created.
///
/// This listener, as it is built from a `PgPool`, supports auto-reconnect. If the active
/// connection being used ever dies, this listener will detect that event, acquire a new connection
/// from the pool, will re-subscribe to all of the originally specified channels, and will resume
/// operations as normal.
pub struct PgPoolListener {
    needs_to_send_listen_cmd: bool,
    connection: Option<PgPoolConnection>,
    channels: Vec<String>,
    pool: PgPool,
}

impl PgPoolListener {
    /// Construct a new instance.
    pub(self) fn new(channels: &[&str], pool: PgPool) -> Self {
        let channels = channels.iter().map(|chan| String::from(*chan)).collect();
        Self {
            needs_to_send_listen_cmd: true,
            connection: None,
            channels,
            pool,
        }
    }
}

impl PgPoolListener {
    /// Receives the next notification available from any of the subscribed channels.
    pub async fn recv(&mut self) -> Result<Option<PgNotification>> {
        loop {
            // Ensure we have an active connection to work with.
            let conn = match &mut self.connection {
                Some(conn) => conn,
                None => {
                    let conn = self.pool.acquire().await?;
                    self.connection = Some(conn);
                    continue;
                }
            };
            // Ensure the current connection has properly registered all listener channels.
            if self.needs_to_send_listen_cmd {
                if let Err(err) = send_listen_query(conn, &self.channels).await {
                    // If we've encountered an error here, test the connection, drop it if needed,
                    // and return the error. The next call to recv will build a new connection if possible.
                    if let Err(_) = conn.ping().await {
                        self.close_conn().await;
                    }
                    return Err(err);
                }
                self.needs_to_send_listen_cmd = false;
            }
            // Await a notification from the DB.
            match conn.receive().await? {
                // We've received an async notification, return it.
                Some(Message::NotificationResponse(notification)) => {
                    return Ok(Some(notification.into()));
                }
                // Protocol error, return the error.
                Some(msg) => {
                    return Err(protocol_err!(
                        "unexpected message received from database {:?}",
                        msg
                    )
                    .into())
                }
                // The connection is dead, ensure that it is dropped, update self state, and loop to try again.
                None => {
                    self.close_conn().await;
                    self.needs_to_send_listen_cmd = true;
                    continue;
                }
            }
        }
    }

    /// Consume this listener, returning a `Stream` of notifications.
    pub fn into_stream(mut self) -> impl Stream<Item = Result<Option<PgNotification>>> {
        stream! {
            loop {
                yield self.recv().await
            }
        }
    }
    /// Close and drop the current connection.
    async fn close_conn(&mut self) {
        if let Some(conn) = self.connection.take() {
            let _ = conn.close().await;
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
