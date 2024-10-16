use core::panic;
use miden_client::{
    accounts::AccountId,
    assets::{Asset, FungibleAsset},
    auth::{StoreAuthenticator, TransactionAuthenticator},
    config::{Endpoint, RpcConfig},
    crypto::{FeltRng, RpoRandomCoin},
    notes::{NoteTag, NoteType},
    rpc::{NodeRpcClient, TonicRpcClient},
    store::{
        sqlite_store::{config::SqliteStoreConfig, SqliteStore},
        InputNoteRecord, NoteFilter, Store,
    },
    transactions::{
        request::{TransactionRequest, TransactionRequestError},
        OutputNote,
    },
    Client, Felt,
};
use miden_lib::notes::create_swap_note;
use rand::{seq::SliceRandom, Rng};
use rusqlite::{Connection, Result};
use std::rc::Rc;

use crate::order::Order;

// Client Setup
// ================================================================================================

pub fn setup_client() -> Client<
    TonicRpcClient,
    RpoRandomCoin,
    SqliteStore,
    StoreAuthenticator<RpoRandomCoin, SqliteStore>,
> {
    let store_config = SqliteStoreConfig::default();
    let store = Rc::new(SqliteStore::new(&store_config).unwrap());
    let mut rng = rand::thread_rng();
    let coin_seed: [u64; 4] = rng.gen();
    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));
    let authenticator = StoreAuthenticator::new_with_rng(store.clone(), rng);
    let rpc_config = RpcConfig {
        endpoint: Endpoint::new("http".to_string(), "localhost".to_string(), 57291),
        timeout_ms: 10000,
    };
    let in_debug_mode = true;
    Client::new(
        TonicRpcClient::new(&rpc_config),
        rng,
        store,
        authenticator,
        in_debug_mode,
    )
}

// Transaction Request Creation
// ================================================================================================

pub fn create_swap_notes_transaction_request(
    num_notes: u8,
    sender: AccountId,
    offering_faucet: AccountId,
    total_asset_offering: u64,
    requesting_faucet: AccountId,
    total_asset_requesting: u64,
    felt_rng: &mut impl FeltRng,
) -> Result<TransactionRequest, TransactionRequestError> {
    // Setup note variables
    let mut expected_future_notes = vec![];
    let mut own_output_notes = vec![];
    let note_type = NoteType::Public;
    let aux = Felt::new(0);

    // Generate random distributions for offering and requesting assets
    let offering_distribution =
        generate_random_distribution(num_notes as usize, total_asset_offering);
    let requesting_distribution =
        generate_random_distribution(num_notes as usize, total_asset_requesting);

    for i in 0..num_notes {
        let offered_asset = Asset::Fungible(
            FungibleAsset::new(offering_faucet, offering_distribution[i as usize]).unwrap(),
        );
        let requested_asset = Asset::Fungible(
            FungibleAsset::new(requesting_faucet, requesting_distribution[i as usize]).unwrap(),
        );

        let (created_note, payback_note_details) = create_swap_note(
            sender,
            offered_asset,
            requested_asset,
            note_type,
            aux,
            felt_rng,
        )?;
        expected_future_notes.push(payback_note_details);
        own_output_notes.push(OutputNote::Full(created_note));
    }

    TransactionRequest::new()
        .with_expected_future_notes(expected_future_notes)
        .with_own_output_notes(own_output_notes)
}

pub fn generate_random_distribution(n: usize, total: u64) -> Vec<u64> {
    if total < n as u64 {
        panic!("Total must at least be equal to n to make sure that all values are non-zero.")
    }

    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(n);
    let mut remaining = total;

    // Generate n-1 random numbers
    for _ in 0..n - 1 {
        if remaining == 0 {
            result.push(1); // Ensure non-zero
            continue;
        }

        let max = remaining.saturating_sub(n as u64 - result.len() as u64 - 1);
        let value = if max > 1 {
            rng.gen_range(1..=(total / n as u64))
        } else {
            1
        };

        result.push(value);
        remaining -= value;
    }

    // Add the last number to make the sum equal to total
    result.push(remaining.max(1));

    // Shuffle the vector to randomize the order
    result.shuffle(&mut rng);

    result
}

pub fn get_notes_by_tag<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
    client: &Client<N, R, S, A>,
    tag: NoteTag,
) -> Vec<InputNoteRecord> {
    let notes = client.get_input_notes(NoteFilter::All).unwrap();

    notes
        .into_iter()
        .filter_map(|note| {
            note.clone().metadata().and_then(|metadata| {
                if metadata.tag() == tag {
                    Some(note)
                } else {
                    None
                }
            })
        })
        .collect()
}

pub fn get_assets_from_swap_note(note: &InputNoteRecord) -> (Asset, Asset) {
    let source_asset =
        Asset::Fungible(note.assets().iter().collect::<Vec<&Asset>>()[0].unwrap_fungible());
    let target_faucet = AccountId::try_from(note.details().inputs()[7]).unwrap();
    let target_amount = note.details().inputs()[4].as_int();
    let target_asset = Asset::Fungible(FungibleAsset::new(target_faucet, target_amount).unwrap());
    (source_asset, target_asset)
}

pub fn print_order_table(title: &str, orders: &[Order]) {
    let mut table = Vec::new();
    table.push("+--------------------------------------------------------------------+--------------------+------------------+--------------------+------------------+----------+".to_string());
    table.push("| Note ID                                                            | Requested Asset    | Amount Requested | Offered Asset      | Offered Amount   | Price    |".to_string());
    table.push("+--------------------------------------------------------------------+--------------------+------------------+--------------------+------------------+----------+".to_string());

    for order in orders {
        let note_id = order
            .id()
            .map_or_else(|| "N/A".to_string(), |id| id.to_string());
        let source_asset_faucet_id = order.source_asset().faucet_id().to_string();
        let source_asset_amount = order.source_asset().unwrap_fungible().amount();
        let target_asset_faucet_id = order.target_asset().faucet_id().to_string();
        let target_asset_amount = order.target_asset().unwrap_fungible().amount();

        table.push(format!(
            "| {:<66} | {:<16} | {:<16} | {:<16} | {:<16} | {:<8.2} |",
            note_id,
            target_asset_faucet_id,
            target_asset_amount,
            source_asset_faucet_id,
            source_asset_amount,
            order.price()
        ));
    }

    table.push("+--------------------------------------------------------------------+--------------------+------------------+--------------------+------------------+----------+".to_string());

    // Print title
    println!("{}\n", title);

    // Print table
    for line in table {
        println!("{}", line);
    }
}

pub fn print_balance_update(orders: &[Order]) {
    if orders.is_empty() {
        println!("No orders to process. Your balance will not change.");
        return;
    }

    let mut total_source_asset = 0u64;
    let mut total_target_asset = 0u64;
    let source_faucet_id = orders[0].target_asset().faucet_id();
    let target_faucet_id = orders[0].source_asset().faucet_id();

    for order in orders {
        total_source_asset += order.target_asset().unwrap_fungible().amount();
        total_target_asset += order.source_asset().unwrap_fungible().amount();
    }

    println!("Balance Update Preview:");
    println!("------------------------");
    println!("Assets you will receive:");
    println!("  Faucet ID: {}", target_faucet_id);
    println!("  Amount: {}", total_target_asset);
    println!("\nAssets you will spend:");
    println!("  Faucet ID: {}", source_faucet_id);
    println!("  Amount: {}", total_source_asset);
    println!("------------------------");
}

pub fn clear_notes_tables(db_path: &str) -> Result<()> {
    // Open a connection to the SQLite database
    let conn = Connection::open(db_path)?;

    // Execute the DELETE commands
    conn.execute_batch(
        "
        DELETE FROM output_notes;
        DELETE FROM input_notes;
    ",
    )?;

    println!("Both output_notes and input_notes tables have been cleared.");

    Ok(())
}
