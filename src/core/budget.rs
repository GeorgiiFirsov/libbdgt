use std::array::TryFromSliceError;
use std::io::Write;

use crate::crypto::{CryptoEngine, CryptoBuffer, Kdf};
use crate::error::{Result, Error};
use crate::sync::{Syncable, SyncEngine};
use crate::datetime::{Clock, Timestamp, JANUARY_1970};
use crate::storage::{EncryptedTransaction, EncryptedAccount, EncryptedCategory, EncryptedPlan, MetaInfo};
use crate::storage::{DataStorage, Id, Transaction, Account, Category, Plan, CategoryType};
use super::config::{Config, InstanceId};
use super::changelog::Changelog;
use super::MALFORMED_TIMESTAMP;


/// Name of income transfer category.
const TRANSFER_INCOME_CAT_NAME: &str = "Transfer (income)";

/// Name of income transfer transaction.
const TRANSFER_INCOME_DESCRIPTION: &str = "--> Transfer (income)";

/// Name of outcome transfer category.
const TRANSFER_OUTCOME_CAT_NAME: &str = "Transfer (outcome)";

/// Name of outcome transfer transaction.
const TRANSFER_OUTCOME_DESCRIPTION: &str = "Transfer (outcome) -->";


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
        // Predefined items creation timestamp is always equal to January 1970
        //

        let origin = self.instance_id();

        self.add_category(&Category { 
            id: Some(St::TRANSFER_INCOME_ID), 
            name: TRANSFER_INCOME_CAT_NAME.to_owned(), 
            category_type: CategoryType::Income,
            meta_info: MetaInfo::new(origin, Some(*JANUARY_1970), None, None)
        })?;

        self.add_category(&Category { 
            id: Some(St::TRANSFER_OUTCOME_ID), 
            name: TRANSFER_OUTCOME_CAT_NAME.to_owned(),
            category_type: CategoryType::Outcome,
            meta_info: MetaInfo::new(origin, Some(*JANUARY_1970), None, None)
        })
    }

    /// Add a new transaction.
    /// 
    /// * `transaction` - transaction data
    pub fn add_transaction(&self, transaction: &Transaction) -> Result<()> {
        //
        // Amount is considered to have a proper sign,
        // so I just add it to a corresponding account's
        // balance.
        // Change timestamp for account should not be 
        // modified in this case, so I don't modify it 
        // in account instance.
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

        self.storage.add_transaction(self.encrypt_transaction(transaction)?)?;
        self.storage.update_account(self.encrypt_account(&decrypted_account)?)?;

        Ok(())
    }

    /// Add transfer transactions.
    /// 
    /// * `amount` - amount of money to transfer between accounts
    /// * `from_account` - account to transfer from
    /// * `to_account` - account to transfer to
    pub fn add_transfer(&self, amount: isize, from_account: Id, to_account: Id, timestamp: Timestamp) -> Result<()> {
        //
        // Transfer can be added only locally, i.e. when syncronization is performed, no notion
        // of transfer exists. Only corresponding transactions are synchronized.
        // Hence, all meta information is filled using reasonable default values.
        //

        let amount = amount.abs();
        let origin = self.instance_id();

        self.add_transaction(&Transaction{
            id: None,
            timestamp: timestamp,
            description: TRANSFER_INCOME_DESCRIPTION.to_owned(),
            account_id: to_account,
            category_id: St::TRANSFER_INCOME_ID,
            amount: amount,
            meta_info: MetaInfo::new(origin, Some(timestamp), None, None)
        })?;

        self.add_transaction(&Transaction{
            id: None,
            timestamp: timestamp,
            description: TRANSFER_OUTCOME_DESCRIPTION.to_owned(),
            account_id: from_account,
            category_id: St::TRANSFER_OUTCOME_ID,
            amount: -amount,
            meta_info: MetaInfo::new(origin, Some(timestamp), None, None)
        })?;

        Ok(())
    }

    /// Remove transaction.
    /// 
    /// * `transaction` - identifier of a transaction to remove
    /// * `emergency` - if `true`, then the linked account will not be updated
    /// * `removal_timestame` - this value will be written as removal timestamp
    pub fn remove_transaction(&self, transaction: Id, emergency: bool, removal_timestamp: Timestamp) -> Result<()> {
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

        self.storage.remove_transaction(transaction, removal_timestamp)
    }

    // Return all transactions.
    pub fn transactions(&self) -> Result<Vec<Transaction>> {
        self.decrypt_transactions(&self.storage.transactions()?)
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
        self.decrypt_transactions(&self.storage.transactions_between(start_timestamp, end_timestamp)?) 
    }

    /// Return all transactions bound with a given account sorted by timestamp 
    /// in descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `account` - account identifier to return transactions for
    pub fn transactions_of(&self, account: Id) -> Result<Vec<Transaction>> {
        self.decrypt_transactions(&self.storage.transactions_of(account)?) 
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
        self.decrypt_transactions(&self.storage.transactions_of_between(account, start_timestamp, end_timestamp)?) 
    }

    /// Return all transactions with given category sorted by timestamp in
    /// descending order.
    /// 
    /// Used for optimization.
    /// 
    /// * `category` - category to return transactions with
    pub fn transactions_with(&self, category: Id) -> Result<Vec<Transaction>> {
        self.decrypt_transactions(&self.storage.transactions_with(category)?) 
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
        self.decrypt_transactions(&self.storage.transactions_with_between(category, start_timestamp, end_timestamp)?) 
    }

    /// Add a new account.
    /// 
    /// * `account` - account data
    pub fn add_account(&self, account: &Account) -> Result<()> {
        self.storage.add_account(self.encrypt_account(account)?)
    }

    /// Remove an account if possible (or forced).
    /// 
    /// If account has transaction and `force` is false, then this function fails.
    /// 
    /// * `account` - identifier of an account to remove
    /// * `force` - if true, then account is deleted anyway with all of its transactions
    /// * `removal_timestame` - this value will be written as removal timestamp
    pub fn remove_account(&self, account: Id, force: bool, removal_timestamp: Timestamp) -> Result<()> {
        if force {
            //
            // Forced removal is requested, hence I need to remove
            // all linked transactions first
            //

            for transaction in self.storage.transactions_of(account)? {
                self.storage.remove_transaction(transaction.id.unwrap(), removal_timestamp)?;
            }
        }

        self.storage.remove_account(account, removal_timestamp)
    }

    /// Return account with a given identifier.
    /// 
    /// * `account` - identifier to return record for
    pub fn account(&self, account: Id) -> Result<Account> {
        self.decrypt_account(&self.storage.account(account)?)
    }

    /// Return all accounts.
    pub fn accounts(&self) -> Result<Vec<Account>> {
        self.decrypt_accounts(&self.storage.accounts()?)
    }

    /// Add a new category.
    /// 
    /// * `category` - category data
    pub fn add_category(&self, category: &Category) -> Result<()> {
        self.storage.add_category(self.encrypt_category(category)?)
    }

    /// Remove category if possible.
    /// 
    /// If there is at leas one transaction with the specified
    /// category, then this function fails. There is no way to
    /// remove category with existing transactions.
    /// 
    /// * `category` - identifier of category to remove
    /// * `removal_timestame` - this value will be written as removal timestamp
    pub fn remove_category(&self, category: Id, removal_timestamp: Timestamp) -> Result<()> {
        self.storage.remove_category(category, removal_timestamp)
    }

    /// Return category with a given identifier.
    /// 
    /// * `category` - identifier to return record for
    pub fn category(&self, category: Id) -> Result<Category> {
        self.decrypt_category(&self.storage.category(category)?)
    }

    /// Return all categories.
    pub fn categories(&self) -> Result<Vec<Category>> {
        self.decrypt_categories(&self.storage.categories()?)
    }

    /// Return all categories of specific type.
    /// 
    /// * `category_type` - type to return categories of
    pub fn categories_of(&self, category_type: CategoryType) -> Result<Vec<Category>> {
        self.decrypt_categories(&self.storage.categories_of(category_type)?)
    }

    /// Add a new plan.
    /// 
    /// * `plan` - plan data
    pub fn add_plan(&self, plan: &Plan) -> Result<()> {
        self.storage.add_plan(self.encrypt_plan(plan)?)
    }

    /// Remove plan.
    /// 
    /// * `plan` - identifier of plan to remove
    /// * `removal_timestame` - this value will be written as removal timestamp
    pub fn remove_plan(&self, plan: Id, removal_timestamp: Timestamp) -> Result<()> {
        self.storage.remove_plan(plan, removal_timestamp)
    }

    /// Return plan with a given identifier.
    /// 
    /// * `plan` - identifier to return record for
    pub fn plan(&self, plan: Id) -> Result<Plan> {
        self.decrypt_plan(&self.storage.plan(plan)?)
    }

    /// Return all plans sorted by category.
    pub fn plans(&self) -> Result<Vec<Plan>> {
        self.decrypt_plans(&self.storage.plans()?)
    }

    /// Return all plans for specific category.
    /// 
    /// * `category` - category to return plans for
    pub fn plans_for(&self, category: Id) -> Result<Vec<Plan>> {
        self.decrypt_plans(&self.storage.plans_for(category)?)
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

    /// Replaces an existsing remote URL with a new one.
    /// 
    /// * `remote` - new remote URL
    pub fn set_remote_url(&self, remote: &str) -> Result<()> {
        self.sync_engine
            .change_remote(remote)
    }
}


impl<Ce, Se, St> Syncable for Budget<Ce, Se, St> 
where
    Ce: CryptoEngine,
    Se: SyncEngine,
    St: DataStorage
{
    type Context = CryptoBuffer;

    type InstanceId = InstanceId;

    fn merge_and_export_changes<Ts, Li, Cl>(&self, timestamp_rw: &mut Ts, last_instance_rw: &mut Li, 
        changelog_rw: &mut Cl, last_sync: &Timestamp, auth: &Self::Context) -> Result<()>
    where
        Ts: std::io::Read + std::io::Write + std::io::Seek,
        Li: std::io::Read + std::io::Write + std::io::Seek,
        Cl: std::io::Read + std::io::Write + std::io::Seek
    {
        let mut cumulative_changelog = if Self::empty_sync_files(timestamp_rw, last_instance_rw, changelog_rw)? {
            //
            // Files are correct, but empty
            // Just return empty changelog
            //

            Changelog::new()
        }
        else {
            //
            // Read remote timestamp and instance identifiers to derive decryption key
            //

            let remote_timestamp = Self::read_timestamp(timestamp_rw)?;
            let remote_instance = Self::read_instance(last_instance_rw)?;

            let remote_salt = Self::make_key_derivation_salt(&remote_timestamp, &remote_instance)?;
            let decryption_key = Kdf::derive_key(auth.as_bytes(), remote_salt.as_bytes(), 
                self.crypto_engine.symmetric_key_length())?;

            //
            // Read and decrypt changelog
            //

            let mut remote_changelog = Vec::new();
            changelog_rw.read_to_end(&mut remote_changelog)?;

            let remote_changelog = self.crypto_engine
                .decrypt_symmetric(decryption_key.as_bytes(), &remote_changelog)?;

            Changelog::from_slice(remote_changelog.as_bytes())?
        };

        //
        // Merge remote and export local changes
        // Then join them together
        //

        let local_changelog = self.export_local_changes(last_sync)?;
        self.merge_changes(&cumulative_changelog, last_sync)?;
        
        cumulative_changelog.append(local_changelog)?;

        //
        // Derive new encryption key, encrypt and write updated values
        //

        let local_timestamp = Clock::now();
        let local_instance = self.instance_id();

        Self::prepare_for_overwrite(timestamp_rw)?;
        Self::write_timestamp(&local_timestamp, timestamp_rw)?;

        Self::prepare_for_overwrite(last_instance_rw)?;
        Self::write_instance(&local_instance, last_instance_rw)?;

        let local_salt = Self::make_key_derivation_salt(&local_timestamp, &local_instance)?;
        let encryption_key = Kdf::derive_key(auth.as_bytes(), local_salt.as_bytes(), 
            self.crypto_engine.symmetric_key_length())?;

        let cumulative_changelog = self.crypto_engine
            .encrypt_symmetric(encryption_key.as_bytes(), &cumulative_changelog.to_vec()?)?;

        Self::prepare_for_overwrite(changelog_rw)?;
        changelog_rw.write_all(cumulative_changelog.as_bytes())?;

        Ok(())
    }
}

impl<Ce, Se, St> Budget<Ce, Se, St>
where
    Ce: CryptoEngine,
    Se: SyncEngine,
    St: DataStorage
{
    fn empty_sync_files<Ts, Li, Cl>(timestamp: &mut Ts, last_instance: &mut Li, changelog: &mut Cl) -> Result<bool>
    where
        Ts: std::io::Seek,
        Li: std::io::Seek,
        Cl: std::io::Seek 
    {
        let seek_position = std::io::SeekFrom::End(0);

        let timestamp_size = timestamp.seek(seek_position)?;
        timestamp.rewind()?;

        let last_instance_size = last_instance.seek(seek_position)?;
        last_instance.rewind()?;

        let changelog_size = changelog.seek(seek_position)?;
        changelog.rewind()?;

        //
        // Either all files are, or timestamp and last instanse are not.
        // Otherwise, files are considered malformed
        //

        match (timestamp_size, last_instance_size, changelog_size) {
            (0, 0, 0) => return Ok(true),
            (1.., 1.., _) => return Ok(false),
            _ => return Err(Error::from_message("msg"))
        };
    }

    fn read_timestamp<R: std::io::Read>(timestamp_reader: &mut R) -> Result<Timestamp> {
        let mut buffer = [0; std::mem::size_of::<i64>()];
        let seconds = match timestamp_reader.read_exact(&mut buffer) {
            Ok(_) => i64::from_le_bytes(buffer),
            _ => 0i64
        };

        Timestamp::from_timestamp(seconds, 0)
            .ok_or(Error::from_message(MALFORMED_TIMESTAMP))
    }

    fn write_timestamp<W: std::io::Write>(timestamp: &Timestamp, timestamp_writer: &mut W) -> Result<()> {
        let timestamp = timestamp
            .timestamp()
            .to_le_bytes();

        timestamp_writer
            .write_all(&timestamp)
            .map_err(Error::from)
    }

    fn read_instance<R: std::io::Read>(last_instance_reader: &mut R) -> Result<InstanceId> {
        let mut buffer = [0; 16];
        last_instance_reader.read_exact(&mut buffer)?;

        Ok(uuid::Uuid::from_bytes(buffer))
    }

    fn write_instance<W: std::io::Write>(instance: &InstanceId, last_instance_writer: &mut W) -> Result<()> {
        last_instance_writer
            .write_all(&instance.into_bytes())
            .map_err(Error::from)
    }

    fn prepare_for_overwrite<S: std::io::Seek>(s: &mut S) -> Result<()> {
        s.rewind()
            .map_err(Error::from)
    }

    fn make_key_derivation_salt(timestamp: &Timestamp, instance: &InstanceId) -> Result<CryptoBuffer> {
        let mut salt = Vec::new();
        salt.write_all(&timestamp.timestamp().to_le_bytes())?;
        salt.write_all(&instance.into_bytes())?;

        Ok(CryptoBuffer::from(salt))
    }

    fn export_local_changes(&self, last_sync: &Timestamp) -> Result<Changelog> {
        let mut local_changelog = Changelog::new();

        //
        // I don't filter out "foreign" items, because it is assumed, that
        // there are none of them since this instance has not been synced
        // during the interval (last_sync, now]
        //

        local_changelog.accounts.added = self.accounts_added_since(*last_sync)?;
        local_changelog.accounts.changed = self.accounts_changed_since(*last_sync)?;
        local_changelog.accounts.removed = self.accounts_removed_since(*last_sync)?;

        local_changelog.categories.added = self.categories_added_since(*last_sync)?;
        local_changelog.categories.changed = self.categories_changed_since(*last_sync)?;
        local_changelog.categories.removed = self.categories_removed_since(*last_sync)?;

        local_changelog.plans.added = self.plans_added_since(*last_sync)?;
        local_changelog.plans.changed = self.plans_changed_since(*last_sync)?;
        local_changelog.plans.removed = self.plans_removed_since(*last_sync)?;

        local_changelog.transactions.added = self.transactions_added_since(*last_sync)?;
        local_changelog.transactions.changed = self.transactions_changed_since(*last_sync)?;
        local_changelog.transactions.removed = self.transactions_removed_since(*last_sync)?;

        Ok(local_changelog)
    }

    fn merge_changes(&self, changelog: &Changelog, last_sync: &Timestamp) -> Result<()> {
        //
        // First, added items are processed in the following order:
        //  1. Accounts
        //  2. Categories
        //  3. Plans
        //  4. Transactions
        //

        self.merge_step(&changelog.accounts.added,
            |account| {
                account.meta_info.added_timestamp.unwrap().ge(last_sync) &&
                account.meta_info.origin != self.instance_id().into_bytes()
            }, 
            |account| {
                //
                // Explicitly set account's balance to its initial value, because
                // they may differ in synced account. It could lead to inconsistency.
                //

                let mut account = account.clone();
                account.balance = account.initial_balance;

                self.add_account(&account)
            }
        )?;

        self.merge_step(&changelog.categories.added,
            |category| {
                category.meta_info.added_timestamp.unwrap().ge(last_sync) &&
                category.meta_info.origin != self.instance_id().into_bytes()
            },
            |category| { self.add_category(category) }
        )?;

        self.merge_step(&changelog.plans.added,
            |plan| {
                plan.meta_info.added_timestamp.unwrap().ge(last_sync) &&
                plan.meta_info.origin != self.instance_id().into_bytes()
            }, 
            |plan| { self.add_plan(plan) }
        )?;

        self.merge_step(&changelog.transactions.added,
            |transaction| {
                transaction.meta_info.added_timestamp.unwrap().ge(last_sync) &&
                transaction.meta_info.origin != self.instance_id().into_bytes()
            },
            |transaction| { self.add_transaction(transaction) }
        )?;

        //
        // Then, changed items are processed in the reverse order
        //

        // For now, no changes can be made, therefore, nothing is processed

        //
        // Finally, removed items are processed in the reverse order too
        //

        self.merge_step(&changelog.transactions.removed,
            |transaction| {
                transaction.meta_info.removed_timestamp.unwrap().ge(last_sync) &&
                transaction.meta_info.origin != self.instance_id().into_bytes()
            },
            |transaction| {
                self.remove_transaction(transaction.id.unwrap(), false,
                    transaction.meta_info.removed_timestamp.unwrap())
            }
        )?;

        self.merge_step(&changelog.plans.removed,
            |plan| {
                plan.meta_info.removed_timestamp.unwrap().ge(last_sync) &&
                plan.meta_info.origin != self.instance_id().into_bytes()
            },
            |plan| {
                self.remove_plan(plan.id.unwrap(), plan.meta_info.removed_timestamp.unwrap())
            }
        )?;

        self.merge_step(&changelog.categories.removed,
            |category| {
                category.meta_info.removed_timestamp.unwrap().ge(last_sync) &&
                category.meta_info.origin != self.instance_id().into_bytes()
            },
            |category| {
                self.remove_category(category.id.unwrap(), category.meta_info.removed_timestamp.unwrap())
            }
        )?;

        self.merge_step(&changelog.accounts.removed,
            |account| {
                account.meta_info.removed_timestamp.unwrap().ge(last_sync) &&
                account.meta_info.origin != self.instance_id().into_bytes()
            },
            |account| {
                self.remove_account(account.id.unwrap(), false,
                    account.meta_info.removed_timestamp.unwrap())
            }
        )?;

        Ok(())
    }

    fn merge_step<T, I, F, Mo>(&self, items: I, filter: F, merge_operation: Mo) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        F: Fn(&T) -> bool,
        Mo: Fn(T) -> Result<()>
    {
        for item in items.into_iter().filter(filter) {
            merge_operation(item)?;
        }

        Ok(())
    }
}


impl<Ce, Se, St> Budget<Ce, Se, St>
where
    Ce: CryptoEngine,
    Se: SyncEngine,
    St: DataStorage
{
    fn transactions_added_since(&self, base: Timestamp) -> Result<Vec<Transaction>> {
        self.decrypt_transactions(&self.storage.transactions_added_since(base)?)
    }

    fn transactions_changed_since(&self, base: Timestamp) -> Result<Vec<Transaction>> {
        self.decrypt_transactions(&self.storage.transactions_changed_since(base)?)
    }

    fn transactions_removed_since(&self, base: Timestamp) -> Result<Vec<Transaction>> {
        self.decrypt_transactions(&self.storage.transactions_removed_since(base)?)
    }

    fn accounts_added_since(&self, base: Timestamp) -> Result<Vec<Account>> {
        self.decrypt_accounts(&self.storage.accounts_added_since(base)?)
    }

    fn accounts_changed_since(&self, base: Timestamp) -> Result<Vec<Account>> {
        self.decrypt_accounts(&self.storage.accounts_changed_since(base)?)
    }

    fn accounts_removed_since(&self, base: Timestamp) -> Result<Vec<Account>> {
        self.decrypt_accounts(&self.storage.accounts_removed_since(base)?)
    }

    fn categories_added_since(&self, base: Timestamp) -> Result<Vec<Category>> {
        self.decrypt_categories(&self.storage.categories_added_since(base)?)
    }

    fn categories_changed_since(&self, base: Timestamp) -> Result<Vec<Category>> {
        self.decrypt_categories(&self.storage.categories_changed_since(base)?)
    }

    fn categories_removed_since(&self, base: Timestamp) -> Result<Vec<Category>> {
        self.decrypt_categories(&self.storage.categories_removed_since(base)?)
    }

    fn plans_added_since(&self, base: Timestamp) -> Result<Vec<Plan>> {
        self.decrypt_plans(&self.storage.plans_added_since(base)?)
    }

    fn plans_changed_since(&self, base: Timestamp) -> Result<Vec<Plan>> {
        self.decrypt_plans(&self.storage.plans_changed_since(base)?)
    }

    fn plans_removed_since(&self, base: Timestamp) -> Result<Vec<Plan>> {
        self.decrypt_plans(&self.storage.plans_removed_since(base)?)
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
            amount: encrypted_amount.as_bytes().into(),
            meta_info: transaction.meta_info
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
            amount: decrypted_amount,
            meta_info: encrypted_transaction.meta_info
        })
    }

    fn decrypt_transactions(&self, encrypted_transactions: &Vec<EncryptedTransaction>) -> Result<Vec<Transaction>> {
        encrypted_transactions
            .iter()
            .map(|transaction| self.decrypt_transaction(transaction))
            .collect()
    }

    fn encrypt_account(&self, account: &Account) -> Result<EncryptedAccount> {
        let encrypted_name = self.encrypt_string(&account.name)?;
        let encrypted_balance = self.encrypt_isize(&account.balance)?;
        let encrypted_initial_balance = self.encrypt_isize(&account.initial_balance)?;

        Ok(EncryptedAccount { 
            id: account.id,
            name: encrypted_name.as_bytes().into(), 
            balance: encrypted_balance.as_bytes().into(),
            initial_balance: encrypted_initial_balance.as_bytes().into(),
            meta_info: account.meta_info
        })
    }

    fn decrypt_account(&self, encrypted_account: &EncryptedAccount) -> Result<Account> {
        let decrypted_name = self.decrypt_string(&encrypted_account.name)?;
        let decrypted_balance = self.decrypt_isize(&encrypted_account.balance)?;
        let decrypted_initial_balance = self.decrypt_isize(&encrypted_account.initial_balance)?;

        Ok(Account { 
            id: encrypted_account.id,
            name: decrypted_name, 
            balance: decrypted_balance,
            initial_balance: decrypted_initial_balance,
            meta_info: encrypted_account.meta_info
        })
    }

    fn decrypt_accounts(&self, encrypted_accounts: &Vec<EncryptedAccount>) -> Result<Vec<Account>> {
        encrypted_accounts
            .iter()
            .map(|account| self.decrypt_account(account))
            .collect()
    }

    fn encrypt_category(&self, category: &Category) -> Result<EncryptedCategory> {
        let encrypted_name = self.encrypt_string(&category.name)?;

        Ok(EncryptedCategory {
            id: category.id,
            name: encrypted_name.as_bytes().into(),
            category_type: category.category_type,
            meta_info: category.meta_info
        })
    }

    fn decrypt_category(&self, encrypted_category: &EncryptedCategory) -> Result<Category> {
        let decrypted_category = self.decrypt_string(&encrypted_category.name)?;

        Ok(Category { 
            id: encrypted_category.id,
            name: decrypted_category, 
            category_type: encrypted_category.category_type,
            meta_info: encrypted_category.meta_info
        })
    }

    fn decrypt_categories(&self, encrypted_categories: &Vec<EncryptedCategory>) -> Result<Vec<Category>> {
        encrypted_categories
            .iter()
            .map(|category| self.decrypt_category(category))
            .collect()
    }

    fn encrypt_plan(&self, plan: &Plan) -> Result<EncryptedPlan> {
        let encrypted_name = self.encrypt_string(&plan.name)?;
        let encrypted_amount_limit = self.encrypt_isize(&plan.amount_limit)?;

        Ok(EncryptedPlan { 
            id: plan.id, 
            category_id: plan.category_id, 
            name: encrypted_name.as_bytes().into(), 
            amount_limit: encrypted_amount_limit.as_bytes().into(),
            meta_info: plan.meta_info
        })
    }

    fn decrypt_plan(&self, encrypted_plan: &EncryptedPlan) -> Result<Plan> {
        let decrypted_name = self.decrypt_string(&encrypted_plan.name)?;
        let decrypted_amount_limit = self.decrypt_isize(&encrypted_plan.amount_limit)?;

        Ok(Plan { 
            id: encrypted_plan.id, 
            category_id: encrypted_plan.category_id, 
            name: decrypted_name, 
            amount_limit: decrypted_amount_limit,
            meta_info: encrypted_plan.meta_info
        })
    }

    fn decrypt_plans(&self, encrypted_plans: &Vec<EncryptedPlan>) -> Result<Vec<Plan>> {
        encrypted_plans
            .iter()
            .map(|plan| self.decrypt_plan(plan))
            .collect()
    }
}
