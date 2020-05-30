use sqlx_cli::Command;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // no special handling here
    sqlx_cli::run(Command::from_args()).await
}
