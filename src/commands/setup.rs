use std::{fs, path::Path, time::Duration};

use clap::Parser;
use miden_client::{
    accounts::{AccountData, AccountStorageType, AccountTemplate},
    assets::{FungibleAsset, TokenSymbol},
    auth::TransactionAuthenticator,
    crypto::FeltRng,
    notes::NoteType,
    rpc::NodeRpcClient,
    store::Store,
    transactions::{build_swap_tag, request::TransactionRequest},
    Client,
};
use tokio::time::sleep;

use crate::{
    constants::{DB_FILE_PATH, USER_ACCOUNT_FILE_PATH},
    utils::{
        create_swap_notes_transaction_request, export_account_data, export_details,
        OrderBookDetails,
    },
};

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
            max_supply: 50,
            storage_type: AccountStorageType::OnChain,
        };
        let (faucet_a, _) = client.new_account(client_template_a).unwrap();
        println!("Created faucet a");

        // Create faucet for AssetB
        let client_template_b = AccountTemplate::FungibleFaucet {
            token_symbol: TokenSymbol::new("ASSETB").unwrap(),
            decimals: 10,
            max_supply: 50,
            storage_type: AccountStorageType::OnChain,
        };
        let (faucet_b, _) = client.new_account(client_template_b).unwrap();
        println!("Created faucet b");

        // Create sender and user accounts
        let client_sender_template = AccountTemplate::BasicWallet {
            mutable_code: false,
            storage_type: AccountStorageType::OnChain,
        };
        let (sender, _) = client.new_account(client_sender_template).unwrap();
        println!("Created sender account");

        let client_user_template = AccountTemplate::BasicWallet {
            mutable_code: false,
            storage_type: AccountStorageType::OnChain,
        };
        let (user, user_seed) = client.new_account(client_user_template).unwrap();
        let user_auth = client.get_account_auth(user.id()).unwrap();
        println!("Created user account");

        // Mint 50 ASSETA directed to sender
        let asset = FungibleAsset::new(faucet_a.id(), 50).unwrap();
        println!("Created fungible asset a");
        let note_type = NoteType::Public;
        let transaction_request =
            TransactionRequest::mint_fungible_asset(asset, sender.id(), note_type, client.rng())
                .unwrap();
        let tx_result = client
            .new_transaction(faucet_a.id(), transaction_request)
            .unwrap();
        let asset_a_note_id = tx_result.relevant_notes()[0].id();
        let _ = client.submit_transaction(tx_result).await.unwrap();

        println!("Minted AssetA and funded sender");

        // Mint 50 ASSETB directed to user
        let asset = FungibleAsset::new(faucet_b.id(), 50).unwrap();
        println!("Created fungible asset b");
        let note_type = NoteType::Public;
        let transaction_request =
            TransactionRequest::mint_fungible_asset(asset, user.id(), note_type, client.rng())
                .unwrap();
        let tx_result = client
            .new_transaction(faucet_b.id(), transaction_request)
            .unwrap();
        let asset_b_note_id = tx_result.relevant_notes()[0].id();
        let _ = client.submit_transaction(tx_result).await.unwrap();

        println!("Minted AssetB and funded user");

        // Sync commited notes
        sleep(Duration::from_secs(20)).await;
        client.sync_state().await.unwrap();

        // Fund sender with ASSETA
        let tx_request = TransactionRequest::consume_notes(vec![asset_a_note_id]);
        let tx_result = client.new_transaction(sender.id(), tx_request).unwrap();
        client.submit_transaction(tx_result).await.unwrap();

        // Fund user with ASSETB
        let tx_request = TransactionRequest::consume_notes(vec![asset_b_note_id]);
        let tx_result = client.new_transaction(user.id(), tx_request).unwrap();
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

        // build swap tag
        let tag = build_swap_tag(NoteType::Public, faucet_a.id(), faucet_b.id()).unwrap();

        // Export order book details
        let details = OrderBookDetails {
            faucet_a: faucet_a.id(),
            faucet_b: faucet_b.id(),
            sender: sender.id(),
            user: user.id(),
            swap_tag: tag,
        };
        export_details(&details).unwrap();

        // Export user data
        let user_data = AccountData::new(user, Some(user_seed), user_auth);
        export_account_data(&user_data, USER_ACCOUNT_FILE_PATH).unwrap();

        // Remove database file
        let file_path = Path::new(DB_FILE_PATH);
        if file_path.exists() {
            println!("Deleting {}", DB_FILE_PATH);
            fs::remove_file(file_path)
                .map_err(|e| format!("Failed to remove file {}: {}", DB_FILE_PATH, e))?;
            println!("File deleted successfully");
        } else {
            println!("{} does not exist", DB_FILE_PATH);
        }

        Ok(())
    }
}
