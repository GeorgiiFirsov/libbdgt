use std::array::TryFromSliceError;

use crate::crypto::{CryptoEngine, KeyIdentifier, CryptoBuffer};
use crate::config::Config;
use crate::error::{Result, Error};
use crate::storage::{EncryptedTransaction, EncryptedAccount, EncryptedCategory};
use super::storage::{DataStorage, Id, Transaction, Account, Category};


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
    /// * `account` - identifier of account to add transaction to
    /// * `transaction` - protected transaction data
    pub fn add_transaction(&self, account: Id, transaction: Transaction) -> Result<()> {
        let encrypted_transaction = self.encrypt_transaction(&transaction)?;
        self.storage.add_transaction(account, encrypted_transaction)
    }

    /// Remove transaction.
    /// 
    /// * `transaction` - identifier of a transaction to remove
    pub fn remove_transaction(&self, transaction: Id) -> Result<()> {
        self.storage.remove_transaction(transaction)
    }

    // Return all transactions, that belong to a specific account.
    /// 
    /// * `account` - identifier of account to list transactions of
    pub fn transactions_of(&self, account: Id) -> Result<Vec<Transaction>> {
        let encrypted_transactions = self.storage.transactions_of(account)?;
        encrypted_transactions
            .iter()
            .map(|t| self.decrypt_transaction(t))
            .collect()
    }

    /// Return all transactions, that have a specific category.
    /// 
    /// * `category` - identifier of category to list transactions with
    pub fn transactions_with(&self, category: Id) -> Result<Vec<Transaction>> {
        let encrypted_transactions: Vec<EncryptedTransaction> = self.storage.transactions_with(category)?;
        encrypted_transactions
            .iter()
            .map(|t| self.decrypt_transaction(t))
            .collect()
    }

    /// Add a new account.
    /// 
    /// * `account` - protected account data
    pub fn add_account(&self, account: Account) -> Result<()> {
        let encrypted_account = self.encrypt_account(&account)?;
        self.storage.add_account(encrypted_account)
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
            .map(|a| self.decrypt_account(a))
            .collect()
    }

    /// Add a new category.
    /// 
    /// * `category` - protected category data
    pub fn add_category(&self, category: Category) -> Result<()> {
        let encrypted_category = self.encrypt_category(&category)?;
        self.storage.add_category(encrypted_category)
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
            .map(|c| self.decrypt_category(c))
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
            datetime: transaction.datetime,
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
            datetime: encrypted_transaction.datetime,
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
}
