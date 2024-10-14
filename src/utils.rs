use core::panic;
use miden_client::{
    accounts::{AccountData, AccountId},
    assets::{Asset, FungibleAsset},
    auth::StoreAuthenticator,
    config::{Endpoint, RpcConfig},
    crypto::{FeltRng, RpoRandomCoin},
    notes::{NoteTag, NoteType},
    rpc::TonicRpcClient,
    store::{
        sqlite_store::{config::SqliteStoreConfig, SqliteStore},
        InputNoteRecord,
    },
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
use rand::{seq::SliceRandom, Rng};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
    rc::Rc,
};

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

    let mut total_offering = 0;
    let mut total_requesting = 0;

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
        total_offering += offering_distribution[i as usize];
        total_requesting += requesting_distribution[i as usize];
        println!(
            "{} - Created note with assets:\noffering: {:?}\nreal offering: {}\nrequesting: {}\nreal requesting: {}\ninputs: {:?}\n",
            i,
            created_note.assets().iter().collect::<Vec<&Asset>>()[0].unwrap_fungible().amount(),
            offering_distribution[i as usize],
            created_note.inputs().values()[4],
            requesting_distribution[i as usize],
            created_note.inputs().values()
        );
        own_output_notes.push(OutputNote::Full(created_note));
    }

    println!("Total generated offering asset: {}", total_offering);
    println!("Total generated requesting asset: {}", total_requesting);

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

// AccountData I/O
// ================================================================================================

pub fn export_account_data(account_data: &AccountData, filename: &str) -> io::Result<()> {
    let serialized = account_data.to_bytes();
    fs::create_dir_all("accounts")?;
    let file_path = Path::new("accounts").join(format!("{}.mac", filename));
    let mut file = File::create(file_path)?;
    file.write_all(&serialized)?;
    Ok(())
}

pub fn import_account_data(file_path: &str) -> io::Result<AccountData> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    AccountData::read_from_bytes(&buffer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

pub fn load_accounts() -> io::Result<Vec<AccountData>> {
    let accounts_dir = Path::new("accounts");

    if !accounts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut accounts = Vec::new();

    for entry in fs::read_dir(accounts_dir)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_str().unwrap();

        match import_account_data(path_str) {
            Ok(account_data) => accounts.push(account_data),
            Err(e) => eprintln!("Error importing account data from {} : {}", path_str, e),
        }
    }

    Ok(accounts)
}

pub fn order_notes(
    tag_a: NoteTag,
    tag_b: NoteTag,
    notes: Vec<InputNoteRecord>,
) -> (
    Vec<InputNoteRecord>,
    Vec<InputNoteRecord>,
    Vec<InputNoteRecord>,
) {
    let mut tag_a_notes = Vec::new();
    let mut tag_b_notes = Vec::new();
    let mut other_notes = Vec::new();
    for note in notes {
        let note_tag = note.metadata().map_or(NoteTag::from(0), |m| m.tag());
        if note_tag == tag_a {
            tag_a_notes.push(note);
        } else if note_tag == tag_b {
            tag_b_notes.push(note);
        } else {
            other_notes.push(note);
        }
    }
    (tag_a_notes, tag_b_notes, other_notes)
}
