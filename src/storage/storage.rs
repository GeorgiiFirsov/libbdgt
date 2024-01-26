use crate::error::Result;
use crate::datetime::Timestamp;
use super::data::{EncryptedTransaction, EncryptedCategory, EncryptedAccount, EncryptedPlan, Id, CategoryType};


/// Storage trait, that provides protected data reading and writing.
///
/// For all data types supported by storage, there are several operations:
///
/// - creation operation. It writes creation timestamp from data meta
///   information if present. Otherwise, current datetime is written.
///
/// - update operation. It does not update any timestamps in storage.
///
/// - update change timestamp. This operation updates change timestamp
///   for a given item.
///
/// - removal operation. It writes removal timestamp if present. Otherwise,
///   current datetime is written.
///
/// - query operation. It does not update any timestamps, just reads all
///   of them.
pub trait DataStorage {
    /// Predefined income transfer category identifier.
    const TRANSFER_INCOME_ID: Id;

    ///Predefined outcome transfer category identifier.
    const TRANSFER_OUTCOME_ID: Id;

    /// Add a new transaction.
    /// 
    /// * `transaction` - protected transaction data
    fn add_transaction(&self, transaction: EncryptedTransaction) -> Result<()>;

    /// Remove transaction.
    /// 
    /// * `transaction` - identifier of a transaction to remove
    fn remove_transaction(&self, transaction: Id) -> Result<()>;

    /// Return transaction with a given identifier.
    /// 
    /// * `transaction` - identifier to return record for
    fn transaction(&self, transaction: Id) -> Result<EncryptedTransaction>;

    /// Return all transactions sorted by timestamp in descending order.
    fn transactions(&self) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions starting from a given time point sorted by 
    /// timestamp in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `start_timestamp` - point in time to start from
    fn transactions_after(&self, start_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions between a given time points (including start 
    /// of the interval and excluding the end) sorted by timestamp in 
    /// descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `start_timestamp` - point in time to start from
    /// * `end_timestamp` - point in time to end before
    fn transactions_between(&self, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions bound with a given account sorted by timestamp 
    /// in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `account` - account identifier to return transactions for
    fn transactions_of(&self, account: Id) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions starting from a given time point bound with 
    /// a given account sorted by timestamp in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `account` - account identifier to return transactions for
    /// * `start_timestamp` - point in time to start from
    fn transactions_of_after(&self, account: Id, start_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions between a given time points (including start 
    /// of the interval and excluding the end) bound with a given account 
    /// sorted by timestamp in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `account` - account identifier to return transactions for
    /// * `start_timestamp` - point in time to start from
    /// * `end_timestamp` - point in time to end before
    fn transactions_of_between(&self, account: Id, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions with given category sorted by timestamp in
    /// descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `category` - category to return transactions with
    fn transactions_with(&self, category: Id) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions starting from a given time point and with 
    /// given category sorted by timestamp in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `category` - category to return transactions with
    /// * `start_timestamp` - point in time to start from
    fn transactions_with_after(&self, category: Id, start_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>>;

    /// Return all transactions between a given time points (including start 
    /// of the interval and excluding the end) and with given category 
    /// sorted by timestamp in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `category` - category to return transactions with
    /// * `start_timestamp` - point in time to start from
    /// * `end_timestamp` - point in time to end before
    fn transactions_with_between(&self, category: Id, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>>;

    /// Add a new account.
    /// 
    /// * `account` - protected account data
    fn add_account(&self, account: EncryptedAccount) -> Result<()>;

    /// Update account.
    /// 
    /// * `account` - account to update (with updated data)
    fn update_account(&self, account: EncryptedAccount) -> Result<()>;

    /// Remove an account if possible (or forced).
    /// 
    /// If account has transaction and `force` is false, then this function fails.
    /// 
    /// * `account` - identifier of an account to remove
    /// * `force` - if true, then account is deleted anyway with all of its transactions
    fn remove_account(&self, account: Id, force: bool) -> Result<()>;

    /// Return account with a given identifier.
    /// 
    /// * `account` - identifier to return record for
    fn account(&self, account: Id) -> Result<EncryptedAccount>;

    /// Return all accounts.
    fn accounts(&self) -> Result<Vec<EncryptedAccount>>;

    /// Add a new category.
    /// 
    /// * `category` - protected category data
    fn add_category(&self, category: EncryptedCategory) -> Result<()>;

    /// Update category.
    /// 
    /// * `category` - category to update (with updated data)
    fn update_category(&self, category: EncryptedCategory) -> Result<()>;

    /// Remove category if possible.
    /// 
    /// If there is at leas one transaction and/or plan with the specified
    /// category, then this function fails. There is no way to
    /// remove category with existing transactions and/or plans.
    /// 
    /// * `category` - identifier of category to remove
    fn remove_category(&self, category: Id) -> Result<()>;

    /// Return category with a given identifier.
    /// 
    /// * `category` - identifier to return record for
    fn category(&self, category: Id) -> Result<EncryptedCategory>;

    /// Return all categories sorted by type.
    fn categories(&self) -> Result<Vec<EncryptedCategory>>;

    /// Return all categories of specific type.
    /// 
    /// * `category_type` - type to return categories of
    fn categories_of(&self, category_type: CategoryType) -> Result<Vec<EncryptedCategory>>;

    /// Add a new plan.
    /// 
    /// * `plan` - protected plan data
    fn add_plan(&self, plan: EncryptedPlan) -> Result<()>;

    /// Update plan.
    /// 
    /// * `plan` - plan to update (with updated data)
    fn update_plan(&self, plan: EncryptedPlan) -> Result<()>;

    /// Remove plan.
    /// 
    /// * `plan` - identifier of plan to remove
    fn remove_plan(&self, plan: Id) -> Result<()>;

    /// Return plan with a given identifier.
    /// 
    /// * `plan` - identifier to return record for
    fn plan(&self, plan: Id) -> Result<EncryptedPlan>;

    /// Return all plans sorted by category.
    fn plans(&self) -> Result<Vec<EncryptedPlan>>;

    /// Return all plans for specific category.
    /// 
    /// * `category` - category to return plans for
    fn plans_for(&self, category: Id) -> Result<Vec<EncryptedPlan>>;

    /// Delete permanently all previously removed items.
    /// 
    /// Actually `remove_*` functions can perform no removal, e.g.
    /// just mark items as removed. This function therefore permanently
    /// deletes such marked items.
    fn clean_removed(&self) -> Result<()>;
}
