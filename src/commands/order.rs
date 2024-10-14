use clap::Parser;

use miden_client::{
    accounts::AccountId, auth::TransactionAuthenticator, crypto::FeltRng, notes::NoteId,
    rpc::NodeRpcClient, store::Store, transactions::request::TransactionRequest, Client,
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Execute and order")]
pub struct OrderCmd {
    // Account executing the order
    account_id: String,

    // Note id
    note_id: String,

    // Note tag
    note_tag: u32,
}

impl OrderCmd {
    pub async fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        mut client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        // Parse account id
        let account_id = AccountId::from_hex(self.account_id.as_str()).unwrap();

        // Parse note id
        let note_id = NoteId::try_from_hex(self.note_id.as_str()).unwrap();

        // Create transaction
        let transaction_request = TransactionRequest::consume_notes(vec![note_id]);

        let transaction = client
            .new_transaction(account_id, transaction_request)
            .map_err(|e| format!("Failed to create transaction: {}", e))?;

        client
            .submit_transaction(transaction)
            .await
            .map_err(|e| format!("Failed to submit transaction: {}", e))?;

        println!("Executed a new order for swap with tag: {}", self.note_tag);

        Ok(())
    }
}
