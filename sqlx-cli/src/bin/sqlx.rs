use clap::Parser;
use console::style;
use sqlx_cli::Opt;

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if !opt.no_dotenv {
        dotenvy::dotenv().ok();
    }

    // no special handling here
    if let Err(error) = sqlx_cli::run(opt).await {
        println!("{} {}", style("error:").bold().red(), error);
        std::process::exit(1);
    }
}
