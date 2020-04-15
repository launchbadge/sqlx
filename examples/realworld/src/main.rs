use async_std::net::ToSocketAddrs;

use sqlx::pool::PoolConnection;
use sqlx_example_realworld::api::{articles, profiles, users};
use sqlx_example_realworld::db;
use sqlx_example_realworld::db::model::{ProvideAuthn, ProvideData};
use sqlx_example_realworld::db::Db;
use tide::middleware::RequestLogger;

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

async fn run_server<S, C>(addr: impl ToSocketAddrs, state: S) -> anyhow::Result<()>
where
    S: Send + Sync + Db<Conn = PoolConnection<C>> + 'static,
    C: sqlx::Connect + ProvideAuthn + ProvideData,
{
    let mut server = tide::with_state(state);

    server.middleware(RequestLogger::new());

    // users
    server.at("/api/users").post(users::register);
    server.at("/api/users/login").post(users::login);
    server
        .at("/api/user")
        .get(users::get_current_user)
        .put(users::update_user);

    // profiles
    server
        .at("/api/profiles/:username")
        .get(profiles::get_profile);
    server
        .at("/api/profiles/:username/follow")
        .post(profiles::follow_user)
        .delete(profiles::unfollow_user);

    // articles
    server
        .at("/api/articles")
        .get(articles::list_articles)
        .post(articles::create_article);

    server
        .at("/api/articles/:slug")
        .get(articles::get_article)
        .put(articles::update_article)
        .delete(articles::delete_article);
    server.at("/api/articles/feed").get(articles::get_feed);

    // favorites
    server
        .at("/api/articles/:slug/favorite")
        .post(articles::favorite_article)
        .delete(articles::unfavorite_article);

    // comments
    server
        .at("/api/articles/:slug/comments")
        .post(articles::add_comment)
        .get(articles::get_comments);
    server
        .at("/api/articles/:slug/comments/:comment_id")
        .delete(articles::delete_comment);

    // tags
    server.at("/api/tags").get(articles::get_tags);

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
