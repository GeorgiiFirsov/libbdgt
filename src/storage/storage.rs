use crate::error::Result;
use super::data::{EncryptedTransaction, EncryptedCategory, EncryptedAccount, Id};


/// Storage trait, that provides protected data reading and writing.
pub trait DataStorage {
    /// Add a new transaction.
    /// 
    /// * `transaction` - protected transaction data
    fn add_transaction(&self, transaction: EncryptedTransaction) -> Result<()>;

    /// Remove transaction.
    /// 
    /// * `transaction` - identifier of a transaction to remove
    fn remove_transaction(&self, transaction: Id) -> Result<()>;

    /// Return all transactions, that belong to a specific account.
    /// 
    /// * `account` - identifier of account to list transactions of
    fn transactions_of(&self, account: Id) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions, that have a specific category.
    /// 
    /// * `category` - identifier of category to list transactions with
    fn transactions_with(&self, category: Id) -> Result<Vec<EncryptedTransaction>>;

    /// Add a new account.
    /// 
    /// * `account` - protected account data
    fn add_account(&self, account: EncryptedAccount) -> Result<()>;

    /// Remove an account if possible (or forced).
    /// 
    /// If account has transaction and `force` is false, then this function fails.
    /// 
    /// * `account` - identifier of an account to remove
    /// * `force` - if true, then account is deleted anyway with all of its transactions
    fn remove_account(&self, account: Id, force: bool) -> Result<()>;

    /// Return all accounts.
    fn accounts(&self) -> Result<Vec<EncryptedAccount>>;

    /// Add a new category.
    /// 
    /// * `category` - protected category data
    fn add_category(&self, category: EncryptedCategory) -> Result<()>;

    /// Remove category if possible.
    /// 
    /// If there is at leas one transaction with the specified
    /// category, then this function fails. There is no way to
    /// remove category with existing transactions.
    /// 
    /// * `category` - identifier of category to remove
    fn remove_category(&self, category: Id) -> Result<()>;

    /// Return all categories.
    fn categories(&self) -> Result<Vec<EncryptedCategory>>;
}
