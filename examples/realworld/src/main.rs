use async_std::net::ToSocketAddrs;

use sqlx_example_realworld::db::model::*;
use sqlx_example_realworld::{api, db};

#[derive(structopt::StructOpt)]
struct Args {
    #[structopt(long, env = "DATABASE_URL")]
    db_url: String,
    #[structopt(short, long, default_value = "localhost")]
    address: String,
    #[structopt(short, long, default_value = "8080")]
    port: u16,
    #[structopt(long, default_value = "sqlite")]
    db: String,
}

async fn run_server<S>(addr: impl ToSocketAddrs, state: S) -> anyhow::Result<()>
where
    S: Send + Sync + ProvideUser + ProvideArticle + 'static,
{
    let mut server = tide::with_state(state);

    server.at("/ping").get(|_| async move { "pong" }); // FIXME(sgg): remove

    server.at("/api/users").post(api::users::register);
    server.at("/api/users/login").post(api::users::login);
    server.at("/api/user").get(api::users::get_current_user);

    server.at("/api/articles").get(api::articles::list_articles);
    server
        .at("/api/articles/:slug")
        .get(api::articles::get_article)
        .post(api::articles::create_article)
        .put(api::articles::update_article);

    server.listen(addr).await?;

    Ok(())
}

async fn _main(args: Args) -> anyhow::Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let Args {
        db_url,
        address,
        port,
        db,
    } = args;

    let addr = (address.as_str(), port);

    match db.as_str() {
        #[cfg(feature = "sqlite")]
        "sqlite" => run_server(addr, db::sqlite::connect(&db_url).await?).await,
        #[cfg(feature = "postgres")]
        "postgres" => run_server(addr, db::pg::connect(&db_url).await?).await,
        other => Err(anyhow::anyhow!(
            "Not compiled with support for DB `{}`",
            other
        )),
    }?;

    Ok(())
}

#[paw::main]
fn main(args: Args) -> anyhow::Result<()> {
    async_std::task::block_on(_main(args))
}
