use clap::{crate_version, AppSettings, FromArgMatches, IntoApp};
use console::style;
use dotenv::dotenv;
use sqlx_cli::Opt;
use std::{env, process};

#[tokio::main]
async fn main() {
    // when invoked as `cargo sqlx [...]` the args we see are `[...]/sqlx-cli sqlx prepare`
    // so we want to notch out that superfluous "sqlx"
    let args = env::args_os().skip(2);

    dotenv().ok();
    let matches = Opt::into_app()
        .version(crate_version!())
        .bin_name("cargo sqlx")
        .setting(AppSettings::NoBinaryName)
        .get_matches_from(args);

    if let Err(error) = sqlx_cli::run(Opt::from_arg_matches(&matches)).await {
        println!("{} {}", style("error:").bold().red(), error);
        process::exit(1);
    }
}
