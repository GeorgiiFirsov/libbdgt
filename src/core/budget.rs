use std::array::TryFromSliceError;

use crate::crypto::{CryptoEngine, CryptoBuffer};
use crate::error::{Result, Error};
use crate::sync::{Syncable, SyncEngine};
use crate::storage::{EncryptedTransaction, EncryptedAccount, EncryptedCategory, EncryptedPlan};
use crate::storage::{DataStorage, Id, Timestamp, Transaction, Account, Category, Plan, CategoryType};
use super::config::{Config, InstanceId};


/// Name of income transfer category.
const TRANSFER_INCOME_CAT_NAME: &str = "Transfer (income)";

/// Name of income transfer transaction.
const TRANSFER_INCOME_DESCRIPTION: &str = "--> Transfer (income)";

/// Name of outcome transfer category.
const TRANSFER_OUTCOME_CAT_NAME: &str = "Transfer (outcome)";

/// Name of outcome transfer transaction.
const TRANSFER_OUTCOME_DESCRIPTION: &str = "Transfer (outcome) -->";


/// Simple changelog representation for some items.
pub struct SimpleChangelog<T> {
    /// Added items.
    pub added: Vec<T>,

    /// Changed items.
    pub changed: Vec<T>,

    /// Removed items.
    pub removed: Vec<T>,
}


/// Database changelog representation.
pub struct Changelog {
    /// Diff for accounts.
    pub accounts: SimpleChangelog<Account>,

    /// Diff for categories.
    pub categories: SimpleChangelog<Category>,

    /// Diff for transactions.
    pub transactions: SimpleChangelog<Transaction>,

    /// Diff for plans.
    pub plans: SimpleChangelog<Plan>,
}


/// Budget manager.
pub struct Budget<Ce, Se, St>
where
    Ce: CryptoEngine,
    Se: SyncEngine,
    St: DataStorage
{
    /// Cryptographic engine used to encrypt sensitive data.
    crypto_engine: Ce,

    /// Syncronization engine.
    sync_engine: Se,

    /// Storage used to store the data.
    storage: St,

    /// Instance configuration.
    config: Config<Ce>,

    /// Key used to encrypt and decrypt sensitive data.
    key: Ce::Key,
}


impl<Ce, Se, St> Budget<Ce, Se, St>
where
    Ce: CryptoEngine,
    Se: SyncEngine,
    St: DataStorage
{
    /// Creates a budget manager instance.
    /// 
    /// * `crypto_engine` - cryptographic engine used to encrypt sensitive data
    /// * `storage` - storage used to store data
    /// * `config` - app's configuration
    pub fn new(crypto_engine: Ce, sync_engine: Se, storage: St, config: Config<Ce>) -> Result<Self> {
        let key = crypto_engine
            .lookup_key(config.key_id())?;

        Ok(Budget { 
            crypto_engine: crypto_engine, 
            sync_engine: sync_engine,
            storage: storage,
            config: config,
            key: key,
        })
    }

    /// Underlying cryptographic engine name.
    pub fn engine(&self) -> &str {
        self.crypto_engine
            .engine()
    }

    /// Underlying cryptofgraphic engine version.
    pub fn engine_version(&self) -> &str {
        self.crypto_engine
            .version()
    }

    /// Encryption key identifier.
    pub fn key_id(&self) -> &Ce::KeyId {
        self.config
            .key_id()
    }

    /// Local instance identifier.
    pub fn instance_id(&self) -> &InstanceId {
        self.config
            .instance_id()
    }

    /// Initializes budget instance for the first time.
    pub fn initialize(&self) -> Result<()> {
        //
        // Add predefined items and ensure, that they have proper identifiers
        //

        self.add_category(Category { 
            id: Some(St::TRANSFER_INCOME_ID), 
            name: TRANSFER_INCOME_CAT_NAME.to_owned(), 
            category_type: CategoryType::Income 
        })?;

        self.add_category(Category { 
            id: Some(St::TRANSFER_OUTCOME_ID), 
            name: TRANSFER_OUTCOME_CAT_NAME.to_owned(),
            category_type: CategoryType::Outcome
        })
    }

    /// Add a new transaction.
    /// 
    /// * `transaction` - transaction data
    pub fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        //
        // Amount is considered to have a proper sign,
        // so I just add it to a corresponding account's
        // balance
        //

        let mut decrypted_account = self.decrypt_account(
            &self.storage.account(transaction.account_id)?)?;

        decrypted_account.balance += transaction.amount;

        //
        // Well... It would be better to use DB's transactions here,
        // but it is more complicated though. 
        // If transaction will not be added, account will not be modified.
        // If account update fails, one can just remove bad transaction
        // with `emergency` flag set to `true`.
        // Hence there is a way to restore consistency.
        //

        self.storage.add_transaction(self.encrypt_transaction(&transaction)?)?;
        self.storage.update_account(self.encrypt_account(&decrypted_account)?)?;

        Ok(())
    }

    /// Add transfer transactions.
    /// 
    /// * `amount` - amount of money to transfer between accounts
    /// * `from_account` - account to transfer from
    /// * `to_account` - account to transfer to
    pub fn add_transfer(&self, amount: isize, from_account: Id, to_account: Id) -> Result<()> {
        let amount = amount.abs();
        let timestamp = chrono::Utc::now();

        self.add_transaction(Transaction{
            id: None,
            timestamp: timestamp.clone(),
            description: TRANSFER_INCOME_DESCRIPTION.to_owned(),
            account_id: to_account,
            category_id: St::TRANSFER_INCOME_ID,
            amount: amount
        })?;

        self.add_transaction(Transaction{
            id: None,
            timestamp: timestamp,
            description: TRANSFER_OUTCOME_DESCRIPTION.to_owned(),
            account_id: from_account,
            category_id: St::TRANSFER_OUTCOME_ID,
            amount: -amount
        })?;

        Ok(())
    }

    /// Remove transaction.
    /// 
    /// * `transaction` - identifier of a transaction to remove
    pub fn remove_transaction(&self, transaction: Id, emergency: bool) -> Result<()> {
        if !emergency {
            //
            // Here is the same story: it would be probably better to use
            // DB's transactions, but it is not the way here.
            // If account is not updated, transaction will not be added.
            // If transaction is not removed, but account is updated yet,
            // one can remove transaction with `emergency` flag set.
            // Hence there is a way to restore consistency.
            //

            let decrypted_transaction = self.decrypt_transaction(
                &self.storage.transaction(transaction)?)?;

            let mut decrypted_account = self.decrypt_account(
                &self.storage.account(decrypted_transaction.account_id)?)?;

            //
            // Again, amount in transaction is considered to have a proper sign,
            // hence I just subtract it from account's balance
            //

            decrypted_account.balance -= decrypted_transaction.amount;

            self.storage.update_account(self.encrypt_account(&decrypted_account)?)?;
        }

        self.storage.remove_transaction(transaction)
    }

    // Return all transactions.
    pub fn transactions(&self) -> Result<Vec<Transaction>> {
        let encrypted_transactions = self.storage.transactions()?;
        encrypted_transactions
            .iter()
            .map(|transaction| self.decrypt_transaction(transaction))
            .collect()
    }

    /// Return all transactions between a given time points (including start 
    /// of the interval and excluding the end) sorted by timestamp in 
    /// descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `start_timestamp` - point in time to start from
    /// * `end_timestamp` - point in time to end before
    pub fn transactions_between(&self, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<Transaction>> {
        let encrypted_transactions = self.storage.transactions_between(start_timestamp, end_timestamp)?;
        encrypted_transactions
            .iter()
            .map(|transaction| self.decrypt_transaction(transaction))
            .collect() 
    }

    /// Return all transactions bound with a given account sorted by timestamp 
    /// in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `account` - account identifier to return transactions for
    pub fn transactions_of(&self, account: Id) -> Result<Vec<Transaction>> {
        let encrypted_transactions = self.storage.transactions_of(account)?;
        encrypted_transactions
            .iter()
            .map(|transaction| self.decrypt_transaction(transaction))
            .collect() 
    }

    /// Return all transactions between a given time points (including start 
    /// of the interval and excluding the end) bound with a given account 
    /// sorted by timestamp in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `account` - account identifier to return transactions for
    /// * `start_timestamp` - point in time to start from
    /// * `end_timestamp` - point in time to end before
    pub fn transactions_of_between(&self, account: Id, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<Transaction>> {
        let encrypted_transactions = self.storage.transactions_of_between(account, start_timestamp, end_timestamp)?;
        encrypted_transactions
            .iter()
            .map(|transaction| self.decrypt_transaction(transaction))
            .collect() 
    }

    /// Return all transactions with given category sorted by timestamp in
    /// descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `category` - category to return transactions with
    pub fn transactions_with(&self, category: Id) -> Result<Vec<Transaction>> {
        let encrypted_transactions = self.storage.transactions_with(category)?;
        encrypted_transactions
            .iter()
            .map(|transaction| self.decrypt_transaction(transaction))
            .collect() 
    }

    /// Return all transactions between a given time points (including start 
    /// of the interval and excluding the end) and with given category 
    /// sorted by timestamp in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `category` - category to return transactions with
    /// * `start_timestamp` - point in time to start from
    /// * `end_timestamp` - point in time to end before
    pub fn transactions_with_between(&self, category: Id, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<Transaction>> {
        let encrypted_transactions = self.storage.transactions_with_between(category, start_timestamp, end_timestamp)?;
        encrypted_transactions
            .iter()
            .map(|transaction| self.decrypt_transaction(transaction))
            .collect() 
    }

    /// Add a new account.
    /// 
    /// * `account` - account data
    pub fn add_account(&self, account: Account) -> Result<()> {
        self.storage.add_account(self.encrypt_account(&account)?)
    }

    /// Remove an account if possible (or forced).
    /// 
    /// If account has transaction and `force` is false, then this function fails.
    /// 
    /// * `account` - identifier of an account to remove
    /// * `force` - if true, then account is deleted anyway with all of its transactions
    pub fn remove_account(&self, account: Id, force: bool) -> Result<()> {
        self.storage.remove_account(account, force)
    }

    /// Return account with a given identifier.
    /// 
    /// * `account` - identifier to return record for
    pub fn account(&self, account: Id) -> Result<Account> {
        let encrypted_account = self.storage.account(account)?;
        self.decrypt_account(&encrypted_account)
    }

    /// Return all accounts.
    pub fn accounts(&self) -> Result<Vec<Account>> {
        let encrypted_accounts = self.storage.accounts()?;
        encrypted_accounts
            .iter()
            .map(|account| self.decrypt_account(account))
            .collect()
    }

    /// Add a new category.
    /// 
    /// * `category` - category data
    pub fn add_category(&self, category: Category) -> Result<()> {
        self.storage.add_category(self.encrypt_category(&category)?)
    }

    /// Remove category if possible.
    /// 
    /// If there is at leas one transaction with the specified
    /// category, then this function fails. There is no way to
    /// remove category with existing transactions.
    /// 
    /// * `category` - identifier of category to remove
    pub fn remove_category(&self, category: Id) -> Result<()> {
        self.storage.remove_category(category)
    }

    /// Return category with a given identifier.
    /// 
    /// * `category` - identifier to return record for
    pub fn category(&self, category: Id) -> Result<Category> {
        let encrypted_category = self.storage.category(category)?;
        self.decrypt_category(&encrypted_category)
    }

    /// Return all categories.
    pub fn categories(&self) -> Result<Vec<Category>> {
        let encrypted_categories = self.storage.categories()?;
        encrypted_categories
            .iter()
            .map(|category| self.decrypt_category(category))
            .collect()
    }

    /// Return all categories of specific type.
    /// 
    /// * `category_type` - type to return categories of
    pub fn categories_of(&self, category_type: CategoryType) -> Result<Vec<Category>> {
        let encrypted_categories = self.storage.categories_of(category_type)?;
        encrypted_categories
            .iter()
            .map(|category| self.decrypt_category(category))
            .collect()
    }

    /// Add a new plan.
    /// 
    /// * `plan` - plan data
    pub fn add_plan(&self, plan: Plan) -> Result<()> {
        self.storage.add_plan(self.encrypt_plan(&plan)?)
    }

    /// Remove plan.
    /// 
    /// * `plan` - identifier of plan to remove
    pub fn remove_plan(&self, plan: Id) -> Result<()> {
        self.storage.remove_plan(plan)
    }

    /// Return plan with a given identifier.
    /// 
    /// * `plan` - identifier to return record for
    pub fn plan(&self, plan: Id) -> Result<Plan> {
        let encrypted_plan = self.storage.plan(plan)?;
        self.decrypt_plan(&encrypted_plan)
    }

    /// Return all plans sorted by category.
    pub fn plans(&self) -> Result<Vec<Plan>> {
        let encrypted_plans = self.storage.plans()?;
        encrypted_plans
            .iter()
            .map(|plan| self.decrypt_plan(plan))
            .collect()
    }

    /// Return all plans for specific category.
    /// 
    /// * `category` - category to return plans for
    pub fn plans_for(&self, category: Id) -> Result<Vec<Plan>> {
        let encrypted_plans = self.storage.plans_for(category)?;
        encrypted_plans
            .iter()
            .map(|plan| self.decrypt_plan(plan))
            .collect()
    }

    /// Delete permanently all previously removed items.
    /// 
    /// Actually `remove_*` functions can perform no removal, e.g.
    /// just mark items as removed. This function therefore permanently
    /// deletes such marked items.
    pub fn clean_removed(&self) -> Result<()> {
        self.storage.clean_removed()
    }

    /// Performs synchronization with remote instances.
    /// 
    /// * `auth` - authentication information for synchronization
    pub fn perform_sync(&self, auth: &[u8]) -> Result<()> {
        //
        // Just use the synchronization engine
        //

        let context = CryptoBuffer::from(auth);
        self.sync_engine
            .perform_sync(self.config.instance_id(), self, &context)?;

        //
        // Some items had been removed since the previous sync,
        // but they were pushed to remote, and now it is not
        // necessary to keep them locally
        //

        self.clean_removed()
    }
}


impl<Ce, Se, St> Syncable for Budget<Ce, Se, St> 
where
    Ce: CryptoEngine,
    Se: SyncEngine,
    St: DataStorage
{
    type Context = CryptoBuffer;

    fn merge_and_export_changes<Ts, Li, Cl>(&self, _timestamp: &mut Ts, _last_instance: &mut Li, _changelog: &mut Cl, _context: &Self::Context) -> Result<()>
    where
        Ts: std::io::Read + std::io::Write,
        Li: std::io::Read + std::io::Write,
        Cl: std::io::Read + std::io::Write 
    {
        // TODO
        Ok(())
    }
}


impl<Ce, Se, St> Budget<Ce, Se, St>
where
    Ce: CryptoEngine,
    Se: SyncEngine,
    St: DataStorage
{
    fn encrypt_string(&self, data: &String) -> Result<CryptoBuffer> {
        self.crypto_engine
            .encrypt(&self.key, data.as_bytes())
    }

    fn decrypt_string(&self, data: &[u8]) -> Result<String> {
        let decrypted = self.crypto_engine
            .decrypt(&self.key, data)?;

        Ok(
            String::from_utf8_lossy(decrypted.as_bytes())
                .to_string()
        )
    }

    fn encrypt_isize(&self, data: &isize) -> Result<CryptoBuffer> {
        self.crypto_engine
            .encrypt(&self.key, &data.to_le_bytes())
    }

    fn decrypt_isize(&self, data: &[u8]) -> Result<isize> {
        let decrypted = self.crypto_engine
            .decrypt(&self.key, data)?;

        let bytes = decrypted
            .as_bytes()
            .try_into()
            .map_err(|e: TryFromSliceError| Error::from_message(e.to_string()))?;

        Ok(isize::from_le_bytes(bytes))
    }

    fn encrypt_transaction(&self, transaction: &Transaction) -> Result<EncryptedTransaction> {
        let encrypted_description = self.encrypt_string(&transaction.description)?;
        let encrypted_amount = self.encrypt_isize(&transaction.amount)?;

        Ok(EncryptedTransaction {
            id: transaction.id,
            timestamp: transaction.timestamp,
            description: encrypted_description.as_bytes().into(),
            account_id: transaction.account_id,
            category_id: transaction.category_id,
            amount: encrypted_amount.as_bytes().into()
        })
    }

    fn decrypt_transaction(&self, encrypted_transaction: &EncryptedTransaction) -> Result<Transaction> {
        let decrypted_description = self.decrypt_string(&encrypted_transaction.description)?;
        let decrypted_amount = self.decrypt_isize(&encrypted_transaction.amount)?;

        Ok(Transaction {
            id: encrypted_transaction.id,
            timestamp: encrypted_transaction.timestamp,
            description: decrypted_description,
            account_id: encrypted_transaction.account_id,
            category_id: encrypted_transaction.category_id,
            amount: decrypted_amount
        })
    }

    fn encrypt_account(&self, account: &Account) -> Result<EncryptedAccount> {
        let encrypted_name = self.encrypt_string(&account.name)?;
        let encrypted_balance = self.encrypt_isize(&account.balance)?;

        Ok(EncryptedAccount { 
            id: account.id,
            name: encrypted_name.as_bytes().into(), 
            balance: encrypted_balance.as_bytes().into() 
        })
    }

    fn decrypt_account(&self, encrypted_account: &EncryptedAccount) -> Result<Account> {
        let decrypted_name = self.decrypt_string(&encrypted_account.name)?;
        let decrypted_balance = self.decrypt_isize(&encrypted_account.balance)?;

        Ok(Account { 
            id: encrypted_account.id,
            name: decrypted_name, 
            balance: decrypted_balance
        })
    }

    fn encrypt_category(&self, category: &Category) -> Result<EncryptedCategory> {
        let encrypted_name = self.encrypt_string(&category.name)?;

        Ok(EncryptedCategory {
            id: category.id,
            name: encrypted_name.as_bytes().into(),
            category_type: category.category_type
        })
    }

    fn decrypt_category(&self, encrypted_category: &EncryptedCategory) -> Result<Category> {
        let decrypted_category = self.decrypt_string(&encrypted_category.name)?;

        Ok(Category { 
            id: encrypted_category.id,
            name: decrypted_category, 
            category_type: encrypted_category.category_type
        })
    }

    fn encrypt_plan(&self, plan: &Plan) -> Result<EncryptedPlan> {
        let encrypted_name = self.encrypt_string(&plan.name)?;
        let encrypted_amount_limit = self.encrypt_isize(&plan.amount_limit)?;

        Ok(EncryptedPlan { 
            id: plan.id, 
            category_id: plan.category_id, 
            name: encrypted_name.as_bytes().into(), 
            amount_limit: encrypted_amount_limit.as_bytes().into()
        })
    }

    fn decrypt_plan(&self, encrypted_plan: &EncryptedPlan) -> Result<Plan> {
        let decrypted_name = self.decrypt_string(&encrypted_plan.name)?;
        let decrypted_amount_limit = self.decrypt_isize(&encrypted_plan.amount_limit)?;

        Ok(Plan { 
            id: encrypted_plan.id, 
            category_id: encrypted_plan.category_id, 
            name: decrypted_name, 
            amount_limit: decrypted_amount_limit 
        })
    }
}
