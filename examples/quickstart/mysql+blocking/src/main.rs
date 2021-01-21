use sqlx::mysql::{MySqlConnectOptions, MySqlConnection};

fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    // start by parsing the connection URL (typically from an environment variable)
    let mut conn: MySqlConnection = MySqlConnectOptions::parse("mysql://root@localhost")?
        // set a password (perhaps from somewhere else than the rest of the URL)
        .password("password")
        // connect to the database (blocking)
        .connect()?;

    // the following are equivalent to the above:

    // let mut conn = MySqlConnection::<Blocking>::connect("mysql://root:password@localhost")?;
    // let mut conn = <MySqlConnection>::connect("mysql://root:password@localhost")?;
    // let mut conn = MySqlConnectOptions::<Blocking>::new().username("root").password("password").connect()?;
    // let mut conn = <MySqlConnectOptions>::new().username("root").password("password").connect()?;

    // the <...> syntax is an escape into the type syntax
    //  when writing a *type*, Rust allows default type parameters
    //  as opposed to writing a *path* where it does not (yet)

    // ping, this makes sure the server is still there
    // hopefully it is - we did just connect to it
    conn.ping()?;

    // close the connection explicitly
    // this kindly informs the database server that we'll be terminating
    // while not strictly required, the server will dispose of connection resources faster
    conn.close()?;

    Ok(())
}
