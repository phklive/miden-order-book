use clap::Parser;
use cli::Cli;

mod cli;
mod commands;
mod utils;

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    cli.execute().await
}
