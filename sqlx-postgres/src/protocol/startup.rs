use sqlx_core::io::Serialize;
use sqlx_core::io::WriteExt;
use sqlx_core::Result;

use crate::io::PgBufMutExt;

// To begin a session, a frontend opens a connection to the server and sends a startup message.
// This message includes the names of the user and of the database the user wants to connect to;
// it also identifies the particular protocol version to be used.

// Optionally, the startup message can include additional settings for run-time parameters.

#[derive(Debug)]
pub struct Startup<'a> {
    /// The database user name to connect as. Required; there is no default.
    pub username: Option<&'a str>,

    /// The database to connect to. Defaults to the user name.
    pub database: Option<&'a str>,

    /// Additional start-up params.
    /// <https://www.postgresql.org/docs/devel/runtime-config-client.html>
    pub params: &'a [(&'a str, &'a str)],
}

impl Serialize<'_, ()> for Startup<'_> {
    fn serialize_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<()> {
        buf.reserve(120);

        buf.write_length_prefixed(|buf| {
            // The protocol version number. The most significant 16 bits are the
            // major version number (3 for the protocol described here). The least
            // significant 16 bits are the minor version number (0
            // for the protocol described here)
            buf.extend(&196_608_i32.to_be_bytes());

            if let Some(username) = self.username {
                // The database user name to connect as.
                encode_startup_param(buf, "user", username);
            }

            if let Some(database) = self.database {
                // The database to connect to. Defaults to the user name.
                encode_startup_param(buf, "database", database);
            }

            for (name, value) in self.params {
                encode_startup_param(buf, name, value);
            }

            // A zero byte is required as a terminator
            // after the last name/value pair.
            buf.push(0);
        });

        Ok(())
    }
}

#[inline]
fn encode_startup_param(buf: &mut Vec<u8>, name: &str, value: &str) {
    buf.write_str_nul(name);
    buf.write_str_nul(value);
}
