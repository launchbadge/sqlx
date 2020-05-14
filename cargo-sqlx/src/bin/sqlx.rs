use cargo_sqlx::Command;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // no special handling here
    cargo_sqlx::run(Command::from_args()).await
}
