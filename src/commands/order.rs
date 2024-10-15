use miden_client::{
    accounts::AccountId,
    assets::{Asset, FungibleAsset},
    auth::TransactionAuthenticator,
    crypto::FeltRng,
    notes::NoteType,
    rpc::NodeRpcClient,
    store::Store,
    transactions::build_swap_tag,
    Client,
};

use clap::Parser;

use crate::{
    order::{match_orders, Order},
    utils::{get_notes_by_tag, print_order_table, sort_orders},
};

#[derive(Debug, Clone, Parser)]
#[command(about = "Execute an order")]
pub struct OrderCmd {
    /// Account executing the order
    account_id: String,

    /// Target faucet id
    target_faucet: String,

    /// Target asset amount
    target_amount: u64,

    /// Source faucet id
    source_faucet: String,

    /// Source asset amount
    source_amount: u64,
}

impl OrderCmd {
    pub async fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        // Parse id's
        let _account_id = AccountId::from_hex(self.account_id.as_str()).unwrap();
        let source_faucet_id = AccountId::from_hex(self.source_faucet.as_str()).unwrap();
        let target_faucet_id = AccountId::from_hex(self.target_faucet.as_str()).unwrap();

        // TODO: add back when fund is fixed
        // // Check if user has balance
        // let (account, _) = client.get_account(account_id).unwrap();
        // if account.vault().get_balance(source_faucet_id).unwrap() < self.source_amount {
        //     panic!("User does not have enough assets to execute this order.");
        // }

        // Build order
        let source_asset =
            Asset::Fungible(FungibleAsset::new(source_faucet_id, self.source_amount).unwrap());
        let target_asset =
            Asset::Fungible(FungibleAsset::new(target_faucet_id, self.target_amount).unwrap());
        let incoming_order = Order::new(None, source_asset, target_asset);

        // Get relevant notes
        let tag = build_swap_tag(NoteType::Public, target_faucet_id, source_faucet_id).unwrap();
        let notes = get_notes_by_tag(client, tag);

        assert!(!notes.is_empty(), "There are no relevant orders available.");

        // find matching orders
        let matching_orders: Vec<Order> = notes
            .into_iter()
            .map(Order::from)
            .filter(|order| match_orders(&incoming_order, order).is_ok())
            .collect();
        let sorted_orders = sort_orders(matching_orders);

        print_order_table(sorted_orders);

        // // find matching orders
        // let matching_order_ids: Result<Vec<NoteId>, OrderError> = relevant_notes
        //     .into_iter()
        //     .map(Order::from)
        //     .filter(|order| match_orders(&incoming_order, order).is_ok())
        //     .map(|matching_order| matching_order.id().ok_or(OrderError::MissingOrderId))
        //     .collect();

        // // Create transaction
        // let transaction_request = TransactionRequest::consume_notes(matching_order_ids);

        // let transaction = client
        //     .new_transaction(account_id, transaction_request)
        //     .map_err(|e| format!("Failed to create transaction: {}", e))?;

        // client
        //     .submit_transaction(transaction)
        //     .await
        //     .map_err(|e| format!("Failed to submit transaction: {}", e))?;

        Ok(())
    }
}
