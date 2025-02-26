mod http;

use accounts::AccountsManager;
use color_eyre::eyre;
use color_eyre::eyre::Context;

#[derive(clap::Parser)]
struct Args {
    #[clap(long, env)]
    database_url: String,

    #[clap(long, env, default_value_t = 0)]
    max_hashing_threads: usize,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let _ = dotenvy::dotenv();

    // (@abonander) I prefer to keep `clap::Parser` fully qualified here because it makes it clear
    // what crate the derive macro is coming from. Otherwise, it requires contextual knowledge
    // to understand that this is parsing CLI arguments.
    let args: Args = clap::Parser::parse();

    tracing_subscriber::fmt::init();

    let pool = sqlx::PgPool::connect(
        // `env::var()` doesn't include the variable name for context like it should.
        &dotenvy::var("DATABASE_URL").wrap_err("DATABASE_URL must be set")?,
    )
    .await
    .wrap_err("could not connect to database")?;

    let max_hashing_threads = if args.max_hashing_threads == 0 {
        std::thread::available_parallelism()
            // We could just default to 1 but that would be a silent pessimization,
            // which would be hard to debug.
            .wrap_err("unable to determine number of available CPU cores; set `--max-hashing-threads` to a nonzero amount")?
            .get()
    } else {
        args.max_hashing_threads
    };

    // Runs migration for `accounts` internally.
    let accounts = AccountsManager::setup(&pool, max_hashing_threads)
        .await
        .wrap_err("error initializing AccountsManager")?;

    payments::migrate(&pool)
        .await
        .wrap_err("error running payments migrations")?;

    // `main()` doesn't actually run from a Tokio worker thread,
    // so spawned tasks hit the global injection queue first and communication with the driver
    // core is always cross-thread.
    //
    // The recommendation is to spawn the `axum::serve` future as a task so it executes directly
    // on a worker thread.

    let http_task = tokio::spawn(http::run(pool, accounts));

    Ok(())
}
