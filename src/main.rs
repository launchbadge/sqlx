#![feature(async_await)]

//use mason::{pg::Connection, ConnectOptions};

#[runtime::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::try_init()?;

    //    let mut conn =
    //        Connection::establish(ConnectOptions::new().user("postgres").password("password")).await?;
    //
    //    conn.execute("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";").await?;
    //
    //    conn.close().await?;

    Ok(())
}
