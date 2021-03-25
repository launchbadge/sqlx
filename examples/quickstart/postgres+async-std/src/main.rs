use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{Close, ConnectOptions, Connection, Executor};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    // start by parsing the connection URL (typically from an environment variable)
    let mut conn: PgConnection = PgConnectOptions::parse("postgres://postgres@localhost")?
        // set a password (perhaps from somewhere else than the rest of the URL)
        .password("password")
        // connect to the database (non-blocking)
        .connect()
        .await?;

    // the following are equivalent to the above:

    // let mut conn = PgConnection::<AsyncStd>::connect("mysql://root:password@localhost").await?;
    // let mut conn = <PgConnection>::connect("mysql://root:password@localhost").await?;
    // let mut conn = PgConnectOptions::<AsyncStd>::new().username("root").password("password").connect().await?;
    // let mut conn = <PgConnectOptions>::new().username("root").password("password").connect().await?;

    // the <...> syntax is an escape into the type syntax
    //  when writing a *type*, Rust allows default type parameters
    //  as opposed to writing a *path* where it does not (yet)

    let res = conn.execute("SELECT 1").await?;

    // ping, this makes sure the server is still there
    // hopefully it is – we did just connect to it
    // conn.ping().await?;

    // close the connection explicitly
    // this kindly informs the database server that we'll be terminating
    // while not strictly required, the server will dispose of connection resources faster
    conn.close().await?;

    Ok(())
}
