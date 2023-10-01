use std::array::TryFromSliceError;

use crate::crypto::{CryptoEngine, KeyIdentifier, CryptoBuffer};
use crate::config::Config;
use crate::error::{Result, Error};
use super::storage::{EncryptedTransaction, EncryptedAccount, EncryptedCategory, EncryptedPlan};
use super::storage::{DataStorage, Id, Transaction, Account, Category, Plan, CategoryType};


/// Budget manager.
pub struct Budget<Ce, St>
where
    Ce: CryptoEngine,
    St: DataStorage
{
    /// Cryptographic engine used to encrypt sensitive data.
    crypto_engine: Ce,

    /// Storage used to store the data.
    storage: St,

    /// Key used to encrypt and decrypt sensitive data.
    key: Ce::Key
}


impl<Ce, St> Budget<Ce, St>
where
    Ce: CryptoEngine,
    St: DataStorage,
    Ce::KeyId: KeyIdentifier
{
    /// Creates a budget manager instance.
    /// 
    /// * `crypto_engine` - cryptographic engine used to encrypt sensitive data
    /// * `storage` - storage used to store data
    /// * `config` - app's configuration
    pub fn new(crypto_engine: Ce, storage: St, config: Config<Ce>) -> Result<Self> {
        let key = crypto_engine
            .lookup_key(config.key_id())?;

        Ok(Budget { 
            crypto_engine: crypto_engine, 
            storage: storage,
            key: key
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

    /// Add a new transaction.
    /// 
    /// * `transaction` - transaction data
    pub fn add_transaction(&self, transaction: Transaction) -> Result<Id> {
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

        let id = self.storage.add_transaction(self.encrypt_transaction(&transaction)?)?;
        self.storage.update_account(self.encrypt_account(&decrypted_account)?)?;

        Ok(id)
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

    /// Add a new account.
    /// 
    /// * `account` - account data
    pub fn add_account(&self, account: Account) -> Result<Id> {
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
    pub fn add_category(&self, category: Category) -> Result<Id> {
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
    pub fn add_plan(&self, plan: Plan) -> Result<Id> {
        self.storage.add_plan(self.encrypt_plan(&plan)?)
    }

    /// Remove plan.
    /// 
    /// * `plan` - identifier of plan to remove
    pub fn remove_plan(&self, plan: Id) -> Result<()> {
        self.storage.remove_plan(plan)
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
}


impl<Ce, St> Budget<Ce, St>
where
    Ce: CryptoEngine,
    St: DataStorage,
    Ce::KeyId: KeyIdentifier
{
    fn encrypt_string(&self, data: &String) -> Result<CryptoBuffer> {
        self.crypto_engine
            .encrypt(&self.key, data.as_bytes())
    }

    fn decrypt_string(&self, data: &[u8]) -> Result<String> {
        let decrypted = self.crypto_engine
            .decrypt(&self.key, data)?;

        Ok(
            String::from_utf8_lossy(decrypted.as_raw())
                .to_string()
        )
    }

    fn encrypt_isize(&self, data: &isize) -> Result<CryptoBuffer> {
        self.crypto_engine
            .encrypt(&self.key, data.to_le_bytes().as_slice())
    }

    fn decrypt_isize(&self, data: &[u8]) -> Result<isize> {
        let decrypted = self.crypto_engine
            .decrypt(&self.key, data)?;

        let bytes = decrypted
            .as_raw()
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
            description: encrypted_description.as_raw().into(),
            category_id: transaction.category_id,
            account_id: transaction.account_id,
            amount: encrypted_amount.as_raw().into()
        })
    }

    fn decrypt_transaction(&self, encrypted_transaction: &EncryptedTransaction) -> Result<Transaction> {
        let decrypted_description = self.decrypt_string(encrypted_transaction.description.as_slice())?;
        let decrypted_amount = self.decrypt_isize(encrypted_transaction.amount.as_slice())?;

        Ok(Transaction {
            id: encrypted_transaction.id,
            timestamp: encrypted_transaction.timestamp,
            description: decrypted_description,
            category_id: encrypted_transaction.category_id,
            account_id: encrypted_transaction.account_id,
            amount: decrypted_amount
        })
    }

    fn encrypt_account(&self, account: &Account) -> Result<EncryptedAccount> {
        let encrypted_name = self.encrypt_string(&account.name)?;
        let encrypted_balance = self.encrypt_isize(&account.balance)?;

        Ok(EncryptedAccount { 
            id: account.id,
            name: encrypted_name.as_raw().into(), 
            balance: encrypted_balance.as_raw().into() 
        })
    }

    fn decrypt_account(&self, encrypted_account: &EncryptedAccount) -> Result<Account> {
        let decrypted_name = self.decrypt_string(encrypted_account.name.as_slice())?;
        let decrypted_balance = self.decrypt_isize(encrypted_account.balance.as_slice())?;

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
            name: encrypted_name.as_raw().into(),
            category_type: category.category_type
        })
    }

    fn decrypt_category(&self, encrypted_category: &EncryptedCategory) -> Result<Category> {
        let decrypted_category = self.decrypt_string(encrypted_category.name.as_slice())?;

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
            name: encrypted_name.as_raw().into(), 
            amount_limit: encrypted_amount_limit.as_raw().into()
        })
    }

    fn decrypt_plan(&self, encrypted_plan: &EncryptedPlan) -> Result<Plan> {
        let decrypted_name = self.decrypt_string(encrypted_plan.name.as_slice())?;
        let decrypted_amount_limit = self.decrypt_isize(encrypted_plan.amount_limit.as_slice())?;

        Ok(Plan { 
            id: encrypted_plan.id, 
            category_id: encrypted_plan.category_id, 
            name: decrypted_name, 
            amount_limit: decrypted_amount_limit 
        })
    }
}
