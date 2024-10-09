use clap::Parser;

use miden_client::{
    auth::TransactionAuthenticator, crypto::FeltRng, rpc::NodeRpcClient, store::Store, Client,
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
        mut _client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        // Import order book details
        // Import user account
        // let account_data = import_account_data(USER_ACCOUNT_FILE_PATH).unwrap();
        // client.import_account(account_data).unwrap();
        // println!("User account has been imported");

        println!("Creating a new {:?} order", self.order_type);
        println!("First leg: {} units of {}", self.amount1, self.faucet_id1);
        println!("Second leg: {} units of {}", self.amount2, self.faucet_id2);
        Ok(())
    }
}
