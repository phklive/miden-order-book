use clap::Parser;
use cli::Cli;

mod cli;
mod commands;
mod constants;
mod errors;
mod order;
mod utils;

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();

    let cli = Cli::parse();

    cli.execute().await
}
