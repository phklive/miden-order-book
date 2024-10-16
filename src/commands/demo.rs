use crate::commands::{
    init::InitCmd, list::ListCmd, order::OrderCmd, query::QueryCmd, setup::SetupCmd,
};
use clap::Parser;
use colored::*;
use log::{info, warn};
use miden_client::{
    auth::TransactionAuthenticator, crypto::FeltRng, rpc::NodeRpcClient, store::Store, Client,
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Demo the full order book flow")]
pub struct DemoCmd {}

impl DemoCmd {
    pub async fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        client: &mut Client<N, R, S, A>,
    ) -> Result<(), String> {
        self.print_cool_start_message();

        info!("Initializing the demo environment...");
        let init = InitCmd {};
        init.execute()
            .map_err(|e| format!("Initialization failed: {}", e))?;

        info!("Setting up the client...");
        let setup = SetupCmd {};
        setup
            .execute(client)
            .await
            .map_err(|e| format!("Setup failed: {}", e))?;

        info!("Importing CLOB data...");
        let clob =
            SetupCmd::import_clob_data().map_err(|e| format!("CLOB data import failed: {}", e))?;

        info!("Querying the network...");
        let query = QueryCmd {
            tags: vec![
                clob.swap_1_2_tag.clone().into(),
                clob.swap_2_1_tag.clone().into(),
            ],
        };
        query
            .execute(client)
            .await
            .map_err(|e| format!("Query failed: {}", e))?;

        info!("Listing available orders...");
        let list = ListCmd {
            tags: vec![clob.swap_1_2_tag.into(), clob.swap_2_1_tag.into()],
        };
        list.execute(client)
            .map_err(|e| format!("Listing failed: {}", e))?;

        info!("Placing a new order...");
        let order = OrderCmd {
            user: clob.user.to_string(),
            target_faucet: clob.faucet1.to_string(),
            target_amount: 10,
            source_faucet: clob.faucet2.to_string(),
            source_amount: 20,
        };
        order
            .execute(client)
            .await
            .map_err(|e| format!("Order placement failed: {}", e))?;

        Ok(())
    }

    fn print_cool_start_message(&self) {
        println!(
            "{}",
            r#"
 __  __ ___ ___  ___  _  _    ___  ___  ___  ___  ___    ___   ___   ___  _  __
|  \/  |_ _|   \| __|| \| |  / _ \| _ \|   \| __|| _ \  | _ ) / _ \ / _ \| |/ /
| |\/| || || |) | _| | .` | | (_) |   /| |) | _| |   /  | _ \| (_) | (_) | ' < 
|_|  |_|___|___/|___||_|\_|  \___/|_|_\|___/|___||_|_\  |___/ \___/ \___/|_|\_\
                                                                              
"#
            .bright_cyan()
        );
        println!("{}", "MIDEN ORDER BOOK".bold().green());
        warn!("Fasten your seatbelts! We're about to take off into the world of decentralized finance!");
    }
}
