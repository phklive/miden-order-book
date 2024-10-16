use clap::Parser;

use miden_client::{
    accounts::{AccountStorageType, AccountTemplate},
    auth::TransactionAuthenticator,
    crypto::FeltRng,
    rpc::NodeRpcClient,
    store::Store,
    Client,
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Create a new user account")]
pub struct LoginCmd {}

impl LoginCmd {
    pub fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        client: &mut Client<N, R, S, A>,
    ) -> Result<(), String> {
        // Create user account
        let wallet_template = AccountTemplate::BasicWallet {
            mutable_code: false,
            storage_type: AccountStorageType::OnChain,
        };

        let (account, _) = client
            .new_account(wallet_template)
            .map_err(|e| e.to_string())?;

        println!("Successful login, account id: {}", account.id());

        Ok(())
    }
}
