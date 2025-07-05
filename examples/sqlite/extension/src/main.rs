use std::str::FromStr;

use sqlx::{
    query,
    sqlite::{SqliteConnectOptions, SqlitePool},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let opts = SqliteConnectOptions::from_str(&std::env::var("DATABASE_URL")?)?
        // The sqlx.toml file controls loading extensions for the CLI
        // and for the query checking macros, *not* for the
        // application while it's running. Thus, if we want the
        // extension to be available during program execution, we need
        // to load it.
        //
        // Note that while in this case the extension path is the same
        // when checking the program (sqlx.toml) and when running it
        // (here), this is not required. The runtime environment can
        // be entirely different from the development one.
        //
        // The extension can be described with a full path, as seen
        // here, but in many cases that will not be necessary. As long
        // as the extension is installed in a directory on the library
        // search path, it is sufficient to just provide the extension
        // name, like "ipaddr"
        .extension("/tmp/sqlite3-lib/ipaddr");

    let db = SqlitePool::connect_with(opts).await?;

    // We're not running the migrations here, for the sake of brevity
    // and to confirm that the needed extension was loaded during the
    // CLI migrate operation. It would not be unusual to run the
    // migrations here as well, though, using the database connection
    // we just configured.

    query!(
        "insert into addresses (address, family) values (?1, ipfamily(?1))",
        "10.0.0.10"
    )
    .execute(&db)
    .await?;

    println!("Query which requires the extension was successfully executed.");

    Ok(())
}
