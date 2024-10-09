use clap::Parser;

use crate::{
    constants::USER_ACCOUNT_FILE_PATH,
    utils::{import_account_data, import_details},
};

use super::sync::SyncCmd;

use miden_client::{
    auth::TransactionAuthenticator, crypto::FeltRng, notes::NoteTag, rpc::NodeRpcClient,
    store::Store, Client,
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Execute and order")]
pub struct OrderCmd {
    /// Type of order: buy or sell
    #[arg(value_enum)]
    order_type: OrderType,

    /// Amount for the first leg of the trade
    amount1: u64,

    /// Faucet ID for the first leg of the trade
    faucet_id1: String,

    /// Amount for the second leg of the trade
    amount2: u64,

    /// Faucet ID for the second leg of the trade
    faucet_id2: String,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OrderType {
    Buy,
    Sell,
}

impl OrderCmd {
    pub async fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        mut client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        // Import order book details
        let order_book_details = import_details().map_err(|e| e.to_string())?;

        // Import user account
        let account_data = import_account_data(USER_ACCOUNT_FILE_PATH).unwrap();
        client.import_account(account_data).unwrap();
        println!("User account has been imported");

        // Query all notes matching a certain tag from rollup
        Self::query(order_book_details.swap_tag, client).await?;

        println!("Creating a new {:?} order", self.order_type);
        println!("First leg: {} units of {}", self.amount1, self.faucet_id1);
        println!("Second leg: {} units of {}", self.amount2, self.faucet_id2);
        Ok(())
    }

    async fn query<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        tag: NoteTag,
        mut client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        client.add_note_tag(tag).map_err(|e| e.to_string())?;

        // Sync rollup state
        let sync_command = SyncCmd {};
        sync_command.execute(client).await?;
        Ok(())
    }
}
