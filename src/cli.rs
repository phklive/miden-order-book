use clap::Parser;

use crate::{
    commands::{init::InitCmd, setup::SetupCmd, sync::SyncCmd},
    utils::setup_client,
};

/// CLI actions
#[derive(Debug, Parser)]
pub enum Command {
    Init(InitCmd),
    Setup(SetupCmd),
    Sync(SyncCmd),
}

/// Root CLI struct
#[derive(Parser, Debug)]
#[clap(
    name = "Miden-order-book",
    about = "Miden order book cli",
    version,
    rename_all = "kebab-case"
)]
pub struct Cli {
    #[clap(subcommand)]
    action: Command,
}

impl Cli {
    pub async fn execute(&self) -> Result<(), String> {
        let client = setup_client();

        // Execute Cli commands
        match &self.action {
            Command::Setup(setup) => setup.execute(client).await,
            Command::Sync(sync) => sync.execute(client).await,
            Command::Init(init) => init.execute(),
        }
    }
}
