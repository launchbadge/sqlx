use clap::Parser;
use console::style;
use dotenv::dotenv;
use sqlx_cli::Opt;
use std::process;

#[derive(Parser, Debug)]
#[clap(bin_name = "cargo")]
enum Cli {
    Sqlx(Opt)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let Cli::Sqlx(opt) = Cli::parse();

    if let Err(error) = sqlx_cli::run(opt).await {
        println!("{} {}", style("error:").bold().red(), error);
        process::exit(1);
    }
}
