use crate::utils::order_notes;
use clap::Parser;
use miden_client::{
    assets::Asset,
    auth::TransactionAuthenticator,
    crypto::FeltRng,
    rpc::NodeRpcClient,
    store::{InputNoteRecord, NoteFilter, Store},
    Client,
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Create a new account and login")]
pub struct ListCmd {
    // BTC/ETH swap tag
    btc_eth_tag: u32,
    // ETH/BTC swap tag
    eth_btc_tag: u32,
}

impl ListCmd {
    pub fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        let notes = client.get_input_notes(NoteFilter::All).unwrap();
        let (btc_eth_notes, eth_btc_notes, _) =
            order_notes(self.btc_eth_tag.into(), self.eth_btc_tag.into(), notes);

        let btc_eth_table = Self::generate_table("BTC/ETH", "BTC", "ETH", &btc_eth_notes);
        let eth_btc_table = Self::generate_table("ETH/BTC", "ETH", "BTC", &eth_btc_notes);

        // Print tables one after another
        for line in btc_eth_table {
            println!("{}", line);
        }

        println!(); // Add a blank line between tables

        for line in eth_btc_table {
            println!("{}", line);
        }

        Ok(())
    }

    fn generate_table(
        title: &str,
        offered_asset: &str,
        requested_asset: &str,
        notes: &[InputNoteRecord],
    ) -> Vec<String> {
        let mut table = Vec::new();
        table.push(format!("{} Notes (total {}):", title, notes.len()));
        table.push(
            "+---------------------------------------------------------------------+---------------+--------+------------------+--------+".to_string(),
        );
        table.push(
            "| Note ID                                                             | Offered Asset | Amount | Requested Asset  | Amount |".to_string(),
        );
        table.push(
            "+---------------------------------------------------------------------+---------------+--------+------------------+--------+".to_string(),
        );
        for note in notes.iter() {
            let (offered_amount, requested_amount) = Self::extract_asset_amounts(note);
            let note_id = format!("{}", note.id());
            table.push(format!(
                "| {:<67} | {:<13} | {:<6} | {:<16} | {:<6} |",
                note_id, offered_asset, offered_amount, requested_asset, requested_amount
            ));
        }
        table.push(
            "+---------------------------------------------------------------------+---------------+--------+------------------+--------+".to_string(),
        );
        table
    }

    fn extract_asset_amounts(note: &InputNoteRecord) -> (String, String) {
        let offered_amount = note.assets().iter().collect::<Vec<&Asset>>()[0]
            .unwrap_fungible()
            .amount()
            .to_string();
        let requested_amount = note.details().inputs()[4].to_string();
        (offered_amount, requested_amount)
    }
}
