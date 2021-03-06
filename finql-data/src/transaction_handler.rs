use super::AssetHandler;
use super::DataError;
use crate::transaction::Transaction;

/// Handler for globally available data of transactions and related data
pub trait TransactionHandler: AssetHandler {
    // insert, get, update and delete for transactions
    fn insert_transaction(&mut self, transaction: &Transaction) -> Result<usize, DataError>;
    fn get_transaction_by_id(&mut self, id: usize) -> Result<Transaction, DataError>;
    fn get_all_transactions(&mut self) -> Result<Vec<Transaction>, DataError>;
    fn update_transaction(&mut self, transaction: &Transaction) -> Result<(), DataError>;
    fn delete_transaction(&mut self, id: usize) -> Result<(), DataError>;
}
