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
use miden_lib::utils::{Deserializable, Serializable};
use rand::{seq::SliceRandom, Rng};

use miden_lib::transaction::TransactionKernel;
use miden_objects::assembly::Assembler;
use miden_objects::{
    notes::{
        Note, NoteAssets, NoteExecutionHint, NoteExecutionMode, NoteInputs, NoteMetadata,
        NoteRecipient, NoteScript,
    },
    NoteError, Word,
};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
    rc::Rc,
};


// Partially Fillable SWAP note
// ================================================================================================

pub fn build_swap_tag(
    note_type: NoteType,
    offered_asset: &Asset,
    requested_asset: &Asset,
) -> Result<NoteTag, NoteError> {
    const SWAP_USE_CASE_ID: u16 = 0;

    // get bits 4..12 from faucet IDs of both assets, these bits will form the tag payload; the
    // reason we skip the 4 most significant bits is that these encode metadata of underlying
    // faucets and are likely to be the same for many different faucets.

    let offered_asset_id: u64 = offered_asset.faucet_id().into();
    let offered_asset_tag = (offered_asset_id >> 52) as u8;

    let requested_asset_id: u64 = requested_asset.faucet_id().into();
    let requested_asset_tag = (requested_asset_id >> 52) as u8;

    let payload = ((offered_asset_tag as u16) << 8) | (requested_asset_tag as u16);

    let execution = NoteExecutionMode::Local;
    match note_type {
        NoteType::Public => NoteTag::for_public_use_case(SWAP_USE_CASE_ID, payload, execution),
        _ => NoteTag::for_local_use_case(SWAP_USE_CASE_ID, payload),
    }
}

/// Generates a SWAP note - swap of assets between two accounts - and returns the note as well as
/// [NoteDetails] for the payback note.
///
/// This script enables a swap of 2 assets between the `sender` account and any other account that
/// is willing to consume the note. The consumer will receive the `offered_asset` and will create a
/// new P2ID note with `sender` as target, containing the `requested_asset`.
///
/// # Errors
/// Returns an error if deserialization or compilation of the `SWAP` script fails.
pub fn create_partial_swap_note(
    creator: AccountId,
    last_consumer: AccountId,
    offered_asset: Asset,
    requested_asset: Asset,
    note_type: NoteType,
    swap_serial_num: [Felt; 4],
    fill_number: u64,
) -> Result<Note, NoteError> {
    let assembler: Assembler = TransactionKernel::assembler_testing().with_debug_mode(false);

    let note_code = include_str!("../src/notes/PUBLIC_SWAPp.masm");
    let note_script = NoteScript::compile(note_code, assembler).unwrap();

    let requested_asset_word: Word = requested_asset.into();
    let tag = build_swap_tag(note_type, &offered_asset, &requested_asset)?;

    let inputs = NoteInputs::new(vec![
        requested_asset_word[0],
        requested_asset_word[1],
        requested_asset_word[2],
        requested_asset_word[3],
        tag.inner().into(),
        Felt::new(0),
        Felt::new(0),
        Felt::new(0),
        Felt::new(fill_number),
        Felt::new(0),
        Felt::new(0),
        Felt::new(0),
        creator.into(),
    ])?;

    let aux = Felt::new(0);

    // build the outgoing note
    let metadata = NoteMetadata::new(
        last_consumer,
        note_type,
        tag,
        NoteExecutionHint::always(),
        aux,
    )?;

    let assets = NoteAssets::new(vec![offered_asset])?;
    let recipient = NoteRecipient::new(swap_serial_num, note_script.clone(), inputs.clone());
    let note = Note::new(assets.clone(), metadata, recipient.clone());

    Ok(note)
}

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
    let mut own_output_notes = vec![];
    let note_type = NoteType::Public;

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

        let swap_serial_num = felt_rng.draw_word();
        let created_note = create_partial_swap_note(
            sender,
            sender,
            offered_asset,
            requested_asset,
            note_type,
            swap_serial_num,
            0,
        )?;
        // expected_future_notes.push(payback_note_details);
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

    TransactionRequest::new().with_own_output_notes(own_output_notes)
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
