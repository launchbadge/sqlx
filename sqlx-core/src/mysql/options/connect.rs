use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::executor::Executor;
use crate::mysql::{MySqlConnectOptions, MySqlConnection};
use futures_core::future::BoxFuture;

impl ConnectOptions for MySqlConnectOptions {
    type Connection = MySqlConnection;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async move {
            let mut conn = MySqlConnection::establish(self).await?;

            // After the connection is established, we initialize by configuring a few
            // connection parameters

            // https://mariadb.com/kb/en/sql-mode/

            // PIPES_AS_CONCAT - Allows using the pipe character (ASCII 124) as string concatenation operator.
            //                   This means that "A" || "B" can be used in place of CONCAT("A", "B").

            // NO_ENGINE_SUBSTITUTION - If not set, if the available storage engine specified by a CREATE TABLE is
            //                          not available, a warning is given and the default storage
            //                          engine is used instead.

            // NO_ZERO_DATE - Don't allow '0000-00-00'. This is invalid in Rust.

            // NO_ZERO_IN_DATE - Don't allow 'YYYY-00-00'. This is invalid in Rust.

            // --

            // Setting the time zone allows us to assume that the output
            // from a TIMESTAMP field is UTC

            // --

            // https://mathiasbynens.be/notes/mysql-utf8mb4

            conn.execute(concat!(
                r#"SET sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT,NO_ENGINE_SUBSTITUTION')),"#,
                r#"time_zone='+00:00',"#,
                r#"NAMES utf8mb4 COLLATE utf8mb4_unicode_ci;"#,
            )).await?;

            Ok(conn)
        })
    }
}
