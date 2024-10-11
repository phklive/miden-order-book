use clap::Parser;

use miden_client::{
    accounts::AccountId,
    auth::TransactionAuthenticator,
    crypto::FeltRng,
    notes::NoteId,
    rpc::NodeRpcClient,
    store::{NoteFilter, Store},
    transactions::request::TransactionRequest,
    Client,
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Execute and order")]
pub struct OrderCmd {
    // Account executing the order
    account_id: String,

    /// Amount for the first leg of the trade
    amount_1: u64,

    /// Amount for the second leg of the trade
    amount_2: u64,

    /// Swap tag
    swap_tag: u32,
}

impl OrderCmd {
    pub async fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        mut client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        // Parse account id
        let account_id = AccountId::from_hex(self.account_id.as_str()).unwrap();

        // Get relevant notes
        let notes = client
            .get_input_notes(NoteFilter::All)
            .map_err(|e| format!("Failed to get input notes: {}", e))?;

        let relevant_note_ids: Vec<NoteId> = notes
            .into_iter()
            .filter(|note| {
                note.metadata()
                    .map(|m| m.tag() == self.swap_tag.into())
                    .unwrap_or(false)
            })
            .map(|note| note.id())
            .take(self.amount_1 as usize)
            .collect();

        if relevant_note_ids.is_empty() {
            return Err("No relevant notes found for the given swap tag".to_string());
        }

        // Create transaction
        let transaction_request = TransactionRequest::consume_notes(relevant_note_ids);

        let transaction = client
            .new_transaction(account_id, transaction_request)
            .map_err(|e| format!("Failed to create transaction: {}", e))?;

        client
            .submit_transaction(transaction)
            .await
            .map_err(|e| format!("Failed to submit transaction: {}", e))?;

        println!("Created a new order for swap with tag: {}", self.swap_tag);
        println!("First leg: {} units", self.amount_1);
        println!("Second leg: {} units", self.amount_2);
        println!("Order has been successfully executed.");

        Ok(())
    }
}
