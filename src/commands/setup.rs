use std::time::Duration;

use clap::Parser;
use miden_client::{
    accounts::{AccountStorageType, AccountTemplate},
    assets::{FungibleAsset, TokenSymbol},
    auth::TransactionAuthenticator,
    crypto::FeltRng,
    notes::NoteType,
    rpc::NodeRpcClient,
    store::Store,
    transactions::request::TransactionRequest,
    Client,
};
use tokio::time::sleep;

use crate::utils::create_swap_notes_transaction_request;

// Setup COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[clap(about = "Setup the order book")]
pub struct SetupCmd {}

impl SetupCmd {
    pub async fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        mut client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        // Create faucet for AssetA
        let client_template_a = AccountTemplate::FungibleFaucet {
            token_symbol: TokenSymbol::new("ASSETA".to_string().as_str()).unwrap(),
            decimals: 10,
            max_supply: 10000,
            storage_type: AccountStorageType::OnChain,
        };
        let (faucet_a, _) = client.new_account(client_template_a).unwrap();
        println!("Created faucet a");

        // Create faucet for AssetB
        let client_template_b = AccountTemplate::FungibleFaucet {
            token_symbol: TokenSymbol::new("ASSETB").unwrap(),
            decimals: 10,
            max_supply: 10000,
            storage_type: AccountStorageType::OnChain,
        };
        let (faucet_b, _) = client.new_account(client_template_b).unwrap();
        println!("Created faucet b");

        // Create sender account
        let client_sender_template = AccountTemplate::BasicWallet {
            mutable_code: false,
            storage_type: AccountStorageType::OnChain,
        };
        let (sender, _) = client.new_account(client_sender_template).unwrap();

        println!("Created sender account");

        // Mint 50 ASSETA directed to sender
        let asset = FungibleAsset::new(faucet_a.id(), 50).unwrap();
        println!("Created fungible asset");
        let note_type = NoteType::Public;
        let transaction_request =
            TransactionRequest::mint_fungible_asset(asset, sender.id(), note_type, client.rng())
                .unwrap();
        let tx_result = client
            .new_transaction(faucet_a.id(), transaction_request)
            .unwrap();
        let asset_note_id = tx_result.relevant_notes()[0].id();
        let _ = client.submit_transaction(tx_result).await.unwrap();

        println!("Minted assets");

        // Sync commited notes
        sleep(Duration::from_secs(10)).await;
        client.sync_state().await.unwrap();

        // Fund sender with ASSETA
        let tx_request = TransactionRequest::consume_notes(vec![asset_note_id]);
        let tx_result = client.new_transaction(sender.id(), tx_request).unwrap();
        client.submit_transaction(tx_result).await.unwrap();

        println!("Applied!");

        // Create 50 swap notes using AssetA and AssetB with same Tag
        let transaction_request = create_swap_notes_transaction_request(
            50,
            sender.id(),
            faucet_a.id(),
            faucet_b.id(),
            client.rng(),
        )
        .unwrap();
        let tx_result = client
            .new_transaction(sender.id(), transaction_request)
            .unwrap();
        client.submit_transaction(tx_result).await.unwrap();

        println!("It worked!");

        Ok(())
    }
}
