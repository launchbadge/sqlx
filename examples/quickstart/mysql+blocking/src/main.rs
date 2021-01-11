use sqlx::mysql::{MySqlConnectOptions, MySqlConnection};
use sqlx::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    // parse the connection URL
    let mut conn: MySqlConnection = MySqlConnectOptions::parse("mysql://root@localhost")?
        // set a password (perhaps from somewhere else than the rest of the URL)
        .password("password")
        // connect to the database (blocking)
        .connect()?;

    // ping, this makes sure the server is still there
    // hopefully it is – we did just connect to it
    conn.ping()?;

    // close the connection explicitly
    // this kindly informs the database server that we'll be terminating
    // while not strictly required, the server will dispose of connection resources faster
    conn.close()?;

    Ok(())
}
