use std::ops::DerefMut;

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;

use crate::connection::Connection;
use crate::describe::Describe;
use crate::executor::Executor;
use crate::pool::PoolConnection;
use crate::postgres::protocol::{Message, NotificationResponse};
use crate::postgres::{PgArguments, PgConnection, PgPool, PgRow, Postgres};
use crate::Result;

type PgPoolConnection = PoolConnection<PgConnection>;

impl PgConnection {
    /// Register this connection as a listener on the specified channel.
    ///
    /// If an error is returned here, the connection will be dropped.
    pub async fn listen(mut self, channel: impl AsRef<str>) -> Result<PgListener<Self>> {
        let cmd = build_listen_all_query(&[channel]);
        let _ = self.send(cmd.as_str()).await?;
        Ok(PgListener::new(self))
    }

    /// Register this connection as a listener on all of the specified channels.
    ///
    /// If an error is returned here, the connection will be dropped.
    pub async fn listen_all(
        mut self,
        channels: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<PgListener<Self>> {
        let cmd = build_listen_all_query(channels);
        let _ = self.send(cmd.as_str()).await?;
        Ok(PgListener::new(self))
    }
}

impl PgPool {
    /// Fetch a new connection from the pool and register it as a listener on the specified channel.
    pub async fn listen(&self, channel: impl AsRef<str>) -> Result<PgListener<PgPoolConnection>> {
        let mut conn = self.acquire().await?;
        let cmd = build_listen_all_query(&[channel]);
        let _ = conn.send(cmd.as_str()).await?;
        Ok(PgListener::new(conn))
    }

    /// Fetch a new connection from the pool and register it as a listener on the specified channels.
    pub async fn listen_all(
        &self,
        channels: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<PgListener<PgPoolConnection>> {
        let mut conn = self.acquire().await?;
        let cmd = build_listen_all_query(channels);
        let _ = conn.send(cmd.as_str()).await?;
        Ok(PgListener::new(conn))
    }
}

impl PgPoolConnection {
    /// Register this connection as a listener on the specified channel.
    ///
    /// If an error is returned here, the connection will be dropped.
    pub async fn listen(mut self, channel: impl AsRef<str>) -> Result<PgListener<Self>> {
        let cmd = build_listen_all_query(&[channel]);
        let _ = self.send(cmd.as_str()).await?;
        Ok(PgListener::new(self))
    }

    /// Register this connection as a listener on all of the specified channels.
    ///
    /// If an error is returned here, the connection will be dropped.
    pub async fn listen_all(
        mut self,
        channels: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<PgListener<Self>> {
        let cmd = build_listen_all_query(channels);
        let _ = self.send(cmd.as_str()).await?;
        Ok(PgListener::new(self))
    }
}

/// A stream of async database notifications.
///
/// Notifications will always correspond to the channel(s) specified this object is created.
pub struct PgListener<C>(C);

impl<C> PgListener<C> {
    /// Construct a new instance.
    pub(self) fn new(conn: C) -> Self {
        Self(conn)
    }
}

impl<C> PgListener<C>
where
    C: DerefMut<Target = PgConnection>,
{
    /// Get the next async notification from the database.
    pub async fn next(&mut self) -> Result<NotifyMessage> {
        loop {
            match (&mut self.0).receive().await? {
                Some(Message::NotificationResponse(notification)) => return Ok(notification.into()),
                Some(msg) => {
                    return Err(protocol_err!(
                        "unexpected message received from database {:?}",
                        msg
                    )
                    .into())
                }
                None => continue,
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
        self.0.close()
    }
}

impl<C> std::ops::Deref for PgListener<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> std::ops::DerefMut for PgListener<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<C: Connection<Database = Postgres>> crate::Executor for PgListener<C> {
    type Database = super::Postgres;

    fn send<'e, 'q: 'e>(&'e mut self, query: &'q str) -> BoxFuture<'e, Result<()>> {
        Box::pin(self.0.send(query))
    }

    fn execute<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: PgArguments,
    ) -> BoxFuture<'e, Result<u64>> {
        Box::pin(self.0.execute(query, args))
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: PgArguments,
    ) -> BoxStream<'e, Result<PgRow>> {
        self.0.fetch(query, args)
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>>> {
        Box::pin(self.0.describe(query))
    }
}

/// An asynchronous message sent from the database.
#[derive(Debug)]
#[non_exhaustive]
pub struct NotifyMessage {
    /// The channel of the notification, which can be thought of as a topic.
    pub channel: String,
    /// The payload of the notification.
    pub payload: String,
}

impl From<Box<NotificationResponse>> for NotifyMessage {
    fn from(src: Box<NotificationResponse>) -> Self {
        Self {
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
