use std::rc::Rc;

use miden_client::{
    accounts::AccountId,
    assets::{Asset, FungibleAsset},
    auth::StoreAuthenticator,
    config::{Endpoint, RpcConfig},
    crypto::{FeltRng, RpoRandomCoin},
    notes::NoteType,
    rpc::TonicRpcClient,
    store::sqlite_store::{config::SqliteStoreConfig, SqliteStore},
    transactions::{
        request::{TransactionRequest, TransactionRequestError},
        OutputNote,
    },
    Client, Felt,
};
use miden_lib::notes::create_swap_note;
use rand::Rng;

pub fn setup_client() -> Client<
    TonicRpcClient,
    RpoRandomCoin,
    SqliteStore,
    StoreAuthenticator<RpoRandomCoin, SqliteStore>,
> {
    let store_config = SqliteStoreConfig::default();
    let store = SqliteStore::new(&store_config).unwrap();
    let store = Rc::new(store);

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

pub fn create_swap_notes_transaction_request(
    num: u8,
    sender: AccountId,
    offering_faucet: AccountId,
    requesting_faucet: AccountId,
    rng: &mut impl FeltRng,
) -> Result<TransactionRequest, TransactionRequestError> {
    // transaction request setup
    let mut expected_future_notes = vec![];
    let mut own_output_notes = vec![];

    // swap note setup
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
