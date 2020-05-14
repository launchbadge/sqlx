use cargo_sqlx::Command;
use structopt::{clap, StructOpt};

use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // when invoked as `cargo sqlx [...]` the args we see are `[...]/cargo-sqlx sqlx prepare`
    // so we want to notch out that superfluous "sqlx"
    let args = env::args_os().skip(2);

    let matches = Command::clap()
        .bin_name("cargo sqlx")
        .setting(clap::AppSettings::NoBinaryName)
        .get_matches_from(args);

    cargo_sqlx::run(Command::from_clap(&matches)).await
}
