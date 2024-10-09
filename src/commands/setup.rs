use std::time::Duration;

use clap::Parser;
use miden_client::{
    accounts::{Account, AccountData, AccountId, AccountStorageType, AccountTemplate},
    assets::{FungibleAsset, TokenSymbol},
    auth::TransactionAuthenticator,
    crypto::FeltRng,
    notes::NoteType,
    rpc::NodeRpcClient,
    store::Store,
    transactions::{build_swap_tag, request::TransactionRequest},
    Client, Word,
};
use tokio::time::sleep;

use crate::utils::{create_swap_notes_transaction_request, export_account_data};

// AccountWithFilename
// ================================================================================================

struct AccountWithFilename {
    account: Account,
    account_seed: Word,
    filename: String,
}

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
        // Create faucet accounts for BTC and ETH
        let (faucet_a, faucet_a_seed) = Self::create_faucet(100, "BTC", &mut client);
        let (faucet_b, faucet_b_seed) = Self::create_faucet(100, "ETH", &mut client);

        // Create admin account
        let (admin, admin_seed) = Self::create_wallet(&mut client);

        // Mint assets for admin
        Self::fund_admin_wallet(faucet_a.id(), faucet_b.id(), admin.id(), &mut client).await;

        // Create 50 BTC/ETH and 50 ETH/BTC swap notes
        Self::create_swap_notes(faucet_a.id(), faucet_b.id(), admin.id(), &mut client).await;

        // Export CLOB data
        let faucet_a_with_filename = AccountWithFilename {
            account: faucet_a,
            account_seed: faucet_a_seed,
            filename: "faucet_a".to_string(),
        };
        let faucet_b_with_filename = AccountWithFilename {
            account: faucet_b,
            account_seed: faucet_b_seed,
            filename: "faucet_b".to_string(),
        };
        let admin_with_filename = AccountWithFilename {
            account: admin,
            account_seed: admin_seed,
            filename: "admin".to_string(),
        };
        Self::export_clob_data(
            faucet_a_with_filename,
            faucet_b_with_filename,
            admin_with_filename,
            &mut client,
        );

        Ok(())
    }

    async fn create_swap_notes<
        N: NodeRpcClient,
        R: FeltRng,
        S: Store,
        A: TransactionAuthenticator,
    >(
        faucet_a: AccountId,
        faucet_b: AccountId,
        admin: AccountId,
        client: &mut Client<N, R, S, A>,
    ) {
        let transaction_request =
            create_swap_notes_transaction_request(50, admin, faucet_a, faucet_b, client.rng())
                .unwrap();
        let tx_result = client.new_transaction(admin, transaction_request).unwrap();
        client.submit_transaction(tx_result).await.unwrap();
    }

    async fn fund_admin_wallet<
        N: NodeRpcClient,
        R: FeltRng,
        S: Store,
        A: TransactionAuthenticator,
    >(
        faucet_a: AccountId,
        faucet_b: AccountId,
        admin: AccountId,
        client: &mut Client<N, R, S, A>,
    ) {
        // setup mint
        let note_type = NoteType::Public;

        // Mint 50 BTC
        let btc = FungibleAsset::new(faucet_a, 50).unwrap();
        let transaction_request =
            TransactionRequest::mint_fungible_asset(btc, admin, note_type, client.rng()).unwrap();
        let tx_result = client
            .new_transaction(faucet_a, transaction_request)
            .unwrap();
        let asset_a_note_id = tx_result.relevant_notes()[0].id();
        client.submit_transaction(tx_result).await.unwrap();

        // Mint 50 ETH
        let eth = FungibleAsset::new(faucet_b, 50).unwrap();
        let transaction_request =
            TransactionRequest::mint_fungible_asset(eth, admin, note_type, client.rng()).unwrap();
        let tx_result = client
            .new_transaction(faucet_b, transaction_request)
            .unwrap();
        let asset_b_note_id = tx_result.relevant_notes()[0].id();
        client.submit_transaction(tx_result).await.unwrap();

        // Sync rollup state
        sleep(Duration::from_secs(20)).await;
        client.sync_state().await.unwrap();

        // Fund receiving wallet
        let tx_request = TransactionRequest::consume_notes(vec![asset_a_note_id, asset_b_note_id]);
        let tx_result = client.new_transaction(admin, tx_request).unwrap();
        client.submit_transaction(tx_result).await.unwrap();
    }

    fn create_wallet<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        client: &mut Client<N, R, S, A>,
    ) -> (Account, Word) {
        let wallet_template = AccountTemplate::BasicWallet {
            mutable_code: false,
            storage_type: AccountStorageType::OnChain,
        };
        client.new_account(wallet_template).unwrap()
    }

    fn create_faucet<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        max_supply: u64,
        token_symbol: &str,
        client: &mut Client<N, R, S, A>,
    ) -> (Account, Word) {
        let faucet_template = AccountTemplate::FungibleFaucet {
            token_symbol: TokenSymbol::new(token_symbol).unwrap(),
            decimals: 10,
            max_supply,
            storage_type: AccountStorageType::OnChain,
        };
        client.new_account(faucet_template).unwrap()
    }

    fn export_clob_data<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        faucet_a: AccountWithFilename,
        faucet_b: AccountWithFilename,
        admin: AccountWithFilename,
        client: &mut Client<N, R, S, A>,
    ) {
        // build swap tags
        let btc_eth_tag = build_swap_tag(
            NoteType::Public,
            faucet_a.account.id(),
            faucet_b.account.id(),
        )
        .unwrap();
        let eth_btc_tag = build_swap_tag(
            NoteType::Public,
            faucet_b.account.id(),
            faucet_a.account.id(),
        )
        .unwrap();

        // Export accounts
        for AccountWithFilename {
            account,
            account_seed,
            filename,
        } in [faucet_a, faucet_b, admin].into_iter()
        {
            let auth = client.get_account_auth(account.id()).unwrap();
            let user_data = AccountData::new(account, Some(account_seed), auth);
            export_account_data(&user_data, filename.as_str()).unwrap();
        }

        // TODO: remove db file

        println!("BTC/ETH tag: {}", btc_eth_tag);
        println!("ETH/BTC tag: {}", eth_btc_tag);
    }
}
