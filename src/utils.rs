use miden_client::{
    accounts::{AccountData, AccountId},
    assets::{Asset, FungibleAsset},
    auth::StoreAuthenticator,
    config::{Endpoint, RpcConfig},
    crypto::{FeltRng, RpoRandomCoin},
    notes::{NoteTag, NoteType},
    rpc::TonicRpcClient,
    store::sqlite_store::{config::SqliteStoreConfig, SqliteStore},
    transactions::{
        request::{TransactionRequest, TransactionRequestError},
        OutputNote,
    },
    Client, Felt,
};
use miden_lib::{
    notes::create_swap_note,
    utils::{Deserializable, Serializable},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read, Write},
    rc::Rc,
};

use crate::constants::DETAILS_FILE_PATH;

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
    num: u8,
    sender: AccountId,
    offering_faucet: AccountId,
    requesting_faucet: AccountId,
    rng: &mut impl FeltRng,
) -> Result<TransactionRequest, TransactionRequestError> {
    let mut expected_future_notes = vec![];
    let mut own_output_notes = vec![];
    let note_type = NoteType::Public;
    let aux = Felt::new(0);
    let offered_asset = Asset::Fungible(FungibleAsset::new(offering_faucet, 1).unwrap());
    let requested_asset = Asset::Fungible(FungibleAsset::new(requesting_faucet, 2).unwrap());

    for _ in 0..num {
        let (created_note, payback_note_details) =
            create_swap_note(sender, offered_asset, requested_asset, note_type, aux, rng)?;
        expected_future_notes.push(payback_note_details);
        own_output_notes.push(OutputNote::Full(created_note));
    }

    println!("Created all notes");
    TransactionRequest::new()
        .with_expected_future_notes(expected_future_notes)
        .with_own_output_notes(own_output_notes)
}

// AccountData I/O
// ================================================================================================

pub fn export_account_data(account_data: &AccountData, file_name: &str) -> io::Result<()> {
    let serialized = account_data.to_bytes();
    let mut file = File::create(file_name)?;
    file.write_all(&serialized)?;
    Ok(())
}

pub fn import_account_data(file_name: &str) -> io::Result<AccountData> {
    let mut file = File::open(file_name)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    AccountData::read_from_bytes(&buffer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

// OrderBookDetails
// ================================================================================================

#[derive(Serialize, Deserialize)]
pub struct OrderBookDetails {
    pub faucet_a: AccountId,
    pub faucet_b: AccountId,
    pub sender: AccountId,
    pub user: AccountId,
    pub swap_tag: NoteTag,
}

pub fn export_details(details: &OrderBookDetails) -> Result<(), io::Error> {
    let serialized =
        toml::to_string(details).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut file = File::create(DETAILS_FILE_PATH)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

pub fn import_details() -> Result<OrderBookDetails, io::Error> {
    let mut file = File::open(DETAILS_FILE_PATH)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let details: OrderBookDetails =
        toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(details)
}
