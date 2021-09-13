use clap::{crate_version, FromArgMatches, IntoApp};
use console::style;
use dotenv::dotenv;
use sqlx_cli::Opt;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let matches = Opt::into_app().version(crate_version!()).get_matches();

    // no special handling here
    if let Err(error) = sqlx_cli::run(Opt::from_arg_matches(&matches)).await {
        println!("{} {}", style("error:").bold().red(), error);
        std::process::exit(1);
    }
}
