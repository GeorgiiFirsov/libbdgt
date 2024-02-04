use crate::location::Location;
use crate::error::{Result, Error};
use crate::datetime::Timestamp;
use super::data::{EncryptedTransaction, EncryptedCategory, EncryptedAccount, EncryptedPlan, Id, CategoryType, MetaInfo};
use super::storage::DataStorage;
use super::{CONSISTENCY_VIOLATION, CANNOT_DELETE_PREDEFINED};


/// Name of DB file.
const DB_FILE: &str = "database";


/// Implementation of [`rusqlite::types::ToSql`] trait for [`CategoryType`].
/// 
/// [`CategoryType::Income`] translates into 0, [`CategoryType::Outcome`] -- into 1.
impl rusqlite::types::ToSql for CategoryType {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let internal_value = match self {
            CategoryType::Income  => 0i64,
            CategoryType::Outcome => 1i64,
        };

        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Integer(internal_value)
        ))
    }
}


/// Implementation of [`rusqlite::types::FromSql`] for [`CategoryType`].
/// 
/// Checks for invalid values in database, translates only valid values.
impl rusqlite::types::FromSql for CategoryType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(CategoryType::Income),
            1 => Ok(CategoryType::Outcome),
            
            // Other integer values are wrong!
            v => Err(rusqlite::types::FromSqlError::OutOfRange(v)),
        }
    }
}


/// Storage implemented using SQLite.
pub struct DbStorage {
    /// Database connection
    db: rusqlite::Connection
} 


impl DbStorage {
    /// Creates a database in provided location.
    /// 
    /// * `loc` - storage location provider
    pub fn create<L: Location>(loc: &L) -> Result<Self> {
        //
        // Create home path if it doesn't exist
        //

        loc.create_if_absent()?;

        //
        // Now I just open DB and create schema
        //

        let storage = Self::open(loc)?;
        storage
            .create_db()
            .and(Ok(storage))
    }

    /// Opens an existing database in provided location.
    /// 
    /// * `loc` - storage location provider
    pub fn open<L: Location>(loc: &L) -> Result<Self> {
        Ok(DbStorage { 
            db: rusqlite::Connection::open(Self::db_path(loc))?
        })
    }
}


impl DataStorage for DbStorage {
    const TRANSFER_INCOME_ID: Id = [0x00; 16];

    const TRANSFER_OUTCOME_ID: Id = [0xFF; 16];

    fn add_transaction(&self, transaction: EncryptedTransaction) -> Result<()> {
        let statement_fmt = match transaction.id {
            None => r#"
                INSERT INTO transactions (timestamp, description, account_id, category_id, amount, _origin, _creation_timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            Some(_) => r#"
                INSERT INTO transactions (transaction_id, timestamp, description, account_id, category_id, amount, _origin, _creation_timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#
        };
        
        match transaction.id {
            None => self.db.execute(statement_fmt, 
                rusqlite::params![transaction.timestamp, transaction.description, transaction.account_id, 
                    transaction.category_id, transaction.amount, transaction.meta_info.origin, 
                    transaction.meta_info.added_timestamp])?,
                
            Some(id) => self.db.execute(statement_fmt, 
                rusqlite::params![id, transaction.timestamp, transaction.description, transaction.account_id, 
                    transaction.category_id, transaction.amount, transaction.meta_info.origin,
                    transaction.meta_info.added_timestamp])?
        };

        Ok(())
    }

    fn remove_transaction(&self, transaction: Id, removal_timestamp: Timestamp) -> Result<()> {
        let statement_fmt = r#"
            UPDATE transactions
               SET _removal_timestamp = ?1
             WHERE transaction_id = ?2
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![removal_timestamp, transaction])?;

        Ok(())
    }

    fn transaction(&self, transaction: Id) -> Result<EncryptedTransaction> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE transaction_id = ?1 AND 
                  _removal_timestamp IS NULL
        "#));

        let mut result = self.query_with_params(statement_fmt, 
            rusqlite::params![transaction], Self::transaction_from_row)?;

        //
        // The only row is returned here
        //

        Ok(result.remove(0))
    }

    fn transactions(&self) -> Result<Vec<EncryptedTransaction>> {
        let statement = Self::select_from_transactions(Some(r#"
            WHERE _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query(statement, Self::transaction_from_row)
    }

    fn transactions_after(&self, start_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE timestamp >= ?1 AND 
                  _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![start_timestamp], Self::transaction_from_row)
    }

    fn transactions_between(&self, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE timestamp >= ?1 AND 
                  timestamp < ?2 AND 
                  _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![start_timestamp, end_timestamp], Self::transaction_from_row)
    }

    fn transactions_of(&self, account: Id) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE account_id = ?1 AND 
                  _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![account], Self::transaction_from_row)
    }

    fn transactions_of_after(&self, account: Id, start_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE account_id = ?1 AND
                  timestamp >= ?2 AND 
                  _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![account, start_timestamp], Self::transaction_from_row)
    }

    fn transactions_of_between(&self, account: Id, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE account_id = ?1 AND
                  timestamp >= ?2 AND
                  timestamp < ?3 AND 
                  _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![account, start_timestamp, end_timestamp], Self::transaction_from_row)
    }

    fn transactions_with(&self, category: Id) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE category_id = ?1 AND 
                  _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![category], Self::transaction_from_row)
    }

    fn transactions_with_after(&self, category: Id, start_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE category_id = ?1 AND
                  timestamp >= ?2 AND 
                  _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![category, start_timestamp], Self::transaction_from_row)
    }

    fn transactions_with_between(&self, category: Id, start_timestamp: Timestamp, end_timestamp: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE category_id = ?1 AND
                  timestamp >= ?2 AND
                  timestamp < ?3 AND 
                  _removal_timestamp IS NULL
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![category, start_timestamp, end_timestamp], Self::transaction_from_row)
    }

    fn transactions_added_since(&self, base: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE _creation_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::transaction_from_row)
    }

    fn transactions_changed_since(&self, base: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE _change_timestamp IS NOT NULL AND
                  _change_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::transaction_from_row)
    }

    fn transactions_removed_since(&self, base: Timestamp) -> Result<Vec<EncryptedTransaction>> {
        let statement_fmt = Self::select_from_transactions(Some(r#"
            WHERE _removal_timestamp IS NOT NULL AND
                  _removal_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::transaction_from_row)
    }

    fn add_account(&self, account: EncryptedAccount) -> Result<()> {
        let statement_fmt = match account.id {
            None => r#"
                INSERT INTO accounts (name, balance, initial_balance, _origin, _creation_timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            Some(_) => r#"
                INSERT INTO accounts (account_id, name, balance, initial_balance, _origin, _creation_timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#
        };

        match account.id {
            None => self.db.execute(statement_fmt, rusqlite::params![account.name, 
                account.balance, account.initial_balance, account.meta_info.origin,
                account.meta_info.added_timestamp])?,

            Some(id) => self.db.execute(statement_fmt, rusqlite::params![id, account.name, 
                account.balance, account.initial_balance, account.meta_info.origin,
                account.meta_info.added_timestamp])?
        };

        Ok(())
    }

    fn update_account(&self, account: EncryptedAccount) -> Result<()> {
        //
        // For now I don't set _change_timestamp here
        // It is reserved for future use
        //

        let statement_fmt = r#"
            UPDATE accounts
               SET name = ?1,
                   balance = ?2
             WHERE account_id = ?3 AND 
                   _removal_timestamp IS NULL
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![account.name, 
                account.balance, account.id])?;

        Ok(())
    }

    fn remove_account(&self, account: Id, removal_timestamp: Timestamp) -> Result<()> {
        //
        // Check if we can delete account: no transaction should belong to it.
        // Only after that I can remove account
        //

        self.ensure_consistency("transactions", "account_id", account)?;

        let statement_fmt = r#"
            UPDATE accounts
               SET _removal_timestamp = ?1
             WHERE account_id = ?2
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![removal_timestamp, account])?;

        Ok(())
    }

    fn account(&self, account: Id) -> Result<EncryptedAccount> {
        let statement_fmt = Self::select_from_accounts(Some(r#"
            WHERE account_id = ?1 AND 
                  _removal_timestamp IS NULL
        "#));

        let mut result = self.query_with_params(statement_fmt, 
            rusqlite::params![account], Self::account_from_row)?;

        //
        // The only row is returned here
        //

        Ok(result.remove(0))
    }

    fn accounts(&self) -> Result<Vec<EncryptedAccount>> {
        let statement = Self::select_from_accounts(Some(r#"
            WHERE _removal_timestamp IS NULL
        "#));

        self.query(statement, Self::account_from_row)
    }

    fn accounts_added_since(&self, base: Timestamp) -> Result<Vec<EncryptedAccount>> {
        let statement_fmt = Self::select_from_accounts(Some(r#"
            WHERE _creation_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::account_from_row)
    }

    fn accounts_changed_since(&self, base: Timestamp) -> Result<Vec<EncryptedAccount>> {
        let statement_fmt = Self::select_from_accounts(Some(r#"
            WHERE _change_timestamp IS NOT NULL AND
                  _change_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::account_from_row)
    }

    fn accounts_removed_since(&self, base: Timestamp) -> Result<Vec<EncryptedAccount>> {
        let statement_fmt = Self::select_from_accounts(Some(r#"
            WHERE _removal_timestamp IS NOT NULL AND
                  _removal_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::account_from_row)
    }

    fn add_category(&self, category: EncryptedCategory) -> Result<()> {
        let statement_fmt = match category.id {
            None => r#"
                    INSERT INTO categories (name, type, _origin, _creation_timestamp)
                    VALUES (?1, ?2, ?3, ?4)
                "#,

            Some(_) => r#"
                    INSERT INTO categories (category_id, name, type, _origin, _creation_timestamp)
                    VALUES (?1, ?2, ?3, ?4, ?5)
                "#
        };

        match category.id {
            None => self.db.execute(statement_fmt, rusqlite::params![category.name, 
                category.category_type, category.meta_info.origin, category.meta_info.added_timestamp])?,

            Some(id) => self.db.execute(statement_fmt, rusqlite::params![id, category.name, 
                category.category_type, category.meta_info.origin, category.meta_info.added_timestamp])?
        };

        Ok(())
    }

    fn remove_category(&self, category: Id, removal_timestamp: Timestamp) -> Result<()> {
        //
        // Check if no transactions and plans reference this category
        //

        if Self::is_predefined_category(category) {
            return Err(Error::from_message(CANNOT_DELETE_PREDEFINED));
        }

        self.ensure_consistency("transactions", "category_id", category)?;
        self.ensure_consistency("plans", "category_id", category)?;

        let statement_fmt = r#"
            UPDATE categories
               SET _removal_timestamp = ?1
             WHERE category_id = ?2
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![removal_timestamp, category])?;

        Ok(())
    }

    fn category(&self, category: Id) -> Result<EncryptedCategory> {
        let statement_fmt = Self::select_from_categories(Some(r#"
            WHERE category_id = ?1 AND 
                  _removal_timestamp IS NULL
        "#));

        let mut result = self.query_with_params(statement_fmt, 
            rusqlite::params![category], Self::category_from_row)?;
        
        //
        // The only row is returned here
        //

        Ok(result.remove(0))
    }

    fn categories(&self) -> Result<Vec<EncryptedCategory>> {
        let statement = Self::select_from_categories(Some(r#"
            WHERE _removal_timestamp IS NULL
            ORDER BY type
        "#));

        self.query(statement, Self::category_from_row)
    }

    fn categories_of(&self, category_type: CategoryType) -> Result<Vec<EncryptedCategory>> {
        let statement_fmt = Self::select_from_categories(Some(r#"
            WHERE type = ?1 AND 
                  _removal_timestamp IS NULL
            ORDER BY type
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![category_type], Self::category_from_row)
    }

    fn categories_added_since(&self, base: Timestamp) -> Result<Vec<EncryptedCategory>> {
        let statement_fmt = Self::select_from_categories(Some(r#"
            WHERE _creation_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::category_from_row)
    }

    fn categories_changed_since(&self, base: Timestamp) -> Result<Vec<EncryptedCategory>> {
        let statement_fmt = Self::select_from_categories(Some(r#"
            WHERE _change_timestamp IS NOT NULL AND
                  _change_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::category_from_row)
    }

    fn categories_removed_since(&self, base: Timestamp) -> Result<Vec<EncryptedCategory>> {
        let statement_fmt = Self::select_from_categories(Some(r#"
            WHERE _removal_timestamp IS NOT NULL AND
                  _removal_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::category_from_row)
    }

    fn add_plan(&self, plan: EncryptedPlan) -> Result<()> {
        let statement_fmt = match plan.id {
            None => r#"
                INSERT INTO plans (category_id, name, amount_limit, _origin, _creation_timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            Some(_) => r#"
                INSERT INTO plans (plan_id, category_id, name, amount_limit, _origin, _creation_timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#
        };

        match plan.id {
            None => self.db.execute(statement_fmt, rusqlite::params![plan.category_id, 
                plan.name, plan.amount_limit, plan.meta_info.origin, plan.meta_info.added_timestamp])?,

            Some(id) => self.db.execute(statement_fmt, rusqlite::params![id, plan.category_id, 
                plan.name, plan.amount_limit, plan.meta_info.origin, plan.meta_info.added_timestamp])?
        };

        Ok(())
    }

    fn remove_plan(&self, plan: Id, removal_timestamp: Timestamp) -> Result<()> {
        let statement_fmt = r#"
            UPDATE plans
               SET _removal_timestamp = ?1
             WHERE plan_id = ?2
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![removal_timestamp, plan])?;

        Ok(())
    }

    fn plan(&self, plan: Id) -> Result<EncryptedPlan> {
        let statement_fmt = Self::select_from_plans(Some(r#"
            WHERE plan_id = ?1 AND 
                  _removal_timestamp IS NULL
        "#));

        let mut result = self.query_with_params(statement_fmt, 
            rusqlite::params![plan], Self::plan_from_row)?;
        
        //
        // The only row is returned here
        //

        Ok(result.remove(0))
    }

    fn plans(&self) -> Result<Vec<EncryptedPlan>> {
        let statement = Self::select_from_plans(Some(r#"
            WHERE _removal_timestamp IS NULL
            ORDER BY category_id
        "#));

        self.query(statement, Self::plan_from_row)
    }

    fn plans_for(&self, category: Id) -> Result<Vec<EncryptedPlan>> {
        let statement_fmt = Self::select_from_plans(Some(r#"
            WHERE category_id = ?1 AND 
                  _removal_timestamp IS NULL
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![category], Self::plan_from_row)
    }

    fn plans_added_since(&self, base: Timestamp) -> Result<Vec<EncryptedPlan>> {
        let statement_fmt = Self::select_from_plans(Some(r#"
            WHERE _creation_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::plan_from_row)
    }

    fn plans_changed_since(&self, base: Timestamp) -> Result<Vec<EncryptedPlan>> {
        let statement_fmt = Self::select_from_plans(Some(r#"
            WHERE _change_timestamp IS NOT NULL AND
                  _change_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::plan_from_row)
    }

    fn plans_removed_since(&self, base: Timestamp) -> Result<Vec<EncryptedPlan>> {
        let statement_fmt = Self::select_from_plans(Some(r#"
            WHERE _removal_timestamp IS NOT NULL AND
                  _removal_timestamp > ?1
            ORDER BY timestamp DESC
        "#));

        self.query_with_params(statement_fmt, rusqlite::params![base], Self::plan_from_row)
    }

    fn clean_removed(&self) -> Result<()> {
        let statement = r#"
            DELETE FROM plans
             WHERE _removal_timestamp IS NOT NULL;

            DELETE FROM transactions
             WHERE _removal_timestamp IS NOT NULL;
            
            DELETE FROM categories
             WHERE _removal_timestamp IS NOT NULL;

            DELETE FROM accounts
             WHERE _removal_timestamp IS NOT NULL;
        "#;

        self.db
            .execute_batch(statement)?;
        
        Ok(())
    }
}


impl DbStorage {
    fn create_db(&self) -> Result<()> {
        //
        // Database will contain table for each entity: transaction, 
        // account, category and plan.
        // For optimization purposes categories table will be
        // additionally indexed by its type, transactions table --
        // by timestamp, plans table -- by category.
        //
        // Each table has two internal columns: `_change_timestamp`
        // and `_removal_timestamp`, that are suitable for syncing
        // content between different instances of the app.
        // All tables are addtionally indexed by mentioned timestamps.
        //

        let create_statement = r#"
            CREATE TABLE accounts (
                account_id          BLOB        PRIMARY KEY DEFAULT (randomblob(16)),
                balance             BYTEA       NOT NULL,
                initial_balance     BYTEA       NOT NULL,
                name                BYTEA       NOT NULL,
                _origin             BYTEA       NOT NULL,
                _creation_timestamp DATETIME    NOT NULL,
                _change_timestamp   DATETIME    NULL,
                _removal_timestamp  DATETIME    NULL
            ) WITHOUT ROWID;

            CREATE INDEX accounts_by_creation_timestamp
                ON accounts (_creation_timestamp);

            CREATE INDEX accounts_by_change_timestamp
                ON accounts (_change_timestamp);

            CREATE INDEX accounts_by_removal_timestamp
                ON accounts (_removal_timestamp);
                
            CREATE TABLE categories (
                category_id         BLOB        PRIMARY KEY DEFAULT (randomblob(16)),
                name                BYTEA       NOT NULL,
                type                TINYINT     NOT NULL,
                _origin             BYTEA       NOT NULL,
                _creation_timestamp DATETIME    NOT NULL,
                _change_timestamp   DATETIME    NULL,
                _removal_timestamp  DATETIME    NULL
            ) WITHOUT ROWID;

            CREATE INDEX categories_by_type
                ON categories (type);

            CREATE INDEX categories_by_creation_timestamp
                ON categories (_creation_timestamp);

            CREATE INDEX categories_by_change_timestamp
                ON categories (_change_timestamp);

            CREATE INDEX categories_by_removal_timestamp
                ON categories (_removal_timestamp);
                
            CREATE TABLE transactions (
                transaction_id      BLOB        PRIMARY KEY DEFAULT (randomblob(16)),
                timestamp           DATETIME    NOT NULL,
                description         BYTEA       NOT NULL,
                account_id          BLOB        REFERENCES accounts(account_id),
                category_id         BLOB        REFERENCES categories(category_id),
                amount              BYTEA       NOT NULL,
                _origin             BYTEA       NOT NULL,
                _creation_timestamp DATETIME    NOT NULL,
                _change_timestamp   DATETIME    NULL,
                _removal_timestamp  DATETIME    NULL
            ) WITHOUT ROWID;

            CREATE INDEX transactions_by_timestamp
                ON transactions (timestamp);

            CREATE INDEX transactions_by_creation_timestamp
                ON transactions (_creation_timestamp);

            CREATE INDEX transactions_by_change_timestamp
                ON transactions (_change_timestamp);

            CREATE INDEX transactions_by_removal_timestamp
                ON transactions (_removal_timestamp);

            CREATE TABLE plans (
                plan_id             BLOB        PRIMARY KEY DEFAULT (randomblob(16)),
                category_id         BLOB        REFERENCES categories(category_id),
                name                BYTEA       NOT NULL,
                amount_limit        BYTEA       NOT NULL,
                _origin             BYTEA       NOT NULL,
                _creation_timestamp DATETIME    NOT NULL,
                _change_timestamp   DATETIME    NULL,
                _removal_timestamp  DATETIME    NULL
            ) WITHOUT ROWID;

            CREATE INDEX plans_by_category
                ON plans (category_id);

            CREATE INDEX plans_by_creation_timestamp
                ON plans (_creation_timestamp);

            CREATE INDEX plans_by_change_timestamp
                ON plans (_change_timestamp);

            CREATE INDEX plans_by_removal_timestamp
                ON plans (_removal_timestamp);
        "#;

        self.db
            .execute_batch(create_statement)
            .map_err(Error::from)
    }

    fn db_path<L: Location>(loc: &L) -> std::path::PathBuf {
        loc.root()
            .join(DB_FILE)
    }

    fn query_with_params<S, T, P, C>(&self, statement: S, params: P, convert: C) -> Result<Vec<T>>
    where
        S: AsRef<str>,
        P: rusqlite::Params,
        C: Fn(&rusqlite::Row<'_>) -> Result<T>
    {
        let mut statement = self.db.prepare(statement.as_ref())?;
        let mut rows = statement.query(params)?;

        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(convert(row)?)
        }

        Ok(result)
    }

    fn query<S, T, C>(&self, statement: S, convert: C) -> Result<Vec<T>>
    where
        S: AsRef<str>,
        C: Fn(&rusqlite::Row<'_>) -> Result<T>
    {
        self.query_with_params(statement, [], convert)
    }

    fn ensure_consistency(&self, table: &str, foreign_key: &str, foreign_key_value: Id) -> Result<()> {
        let statement_fmt = format!(r#"
            SELECT COUNT(*) FROM {}
             WHERE _removal_timestamp IS NULL
               AND {} = ?1
            "#, table, foreign_key);

        let count: usize = self.db
            .query_row(statement_fmt.as_str(), rusqlite::params![foreign_key_value], 
                |row| row.get(0))?;

        if 0 < count {
            return Err(Error::from_message_with_extra(CONSISTENCY_VIOLATION,
                format!("Table: {}, foreign key: {}", table, foreign_key)));
        }

        Ok(())
    }

    fn is_predefined_category(category: Id) -> bool {
        let predefined = [
            Self::TRANSFER_INCOME_ID,
            Self::TRANSFER_OUTCOME_ID
        ];

        predefined.contains(&category)
    }
}


impl DbStorage {
    fn select_from_transactions<S: Into<String>>(modifiers: Option<S>) -> String {
        let modifiers = modifiers
            .map_or(String::new(), S::into);

        return format!(r#"
            SELECT transaction_id, timestamp, description, account_id, category_id, amount, 
                   _origin, _creation_timestamp, _change_timestamp, _removal_timestamp
              FROM transactions
                {}
        "#, modifiers);
    }

    fn select_from_accounts<S: Into<String>>(modifiers: Option<S>) -> String {
        let modifiers = modifiers
            .map_or(String::new(), S::into);

        return format!(r#"
            SELECT account_id, name, balance, initial_balance, _origin, _creation_timestamp, _change_timestamp, _removal_timestamp
              FROM accounts
                {}
        "#, modifiers);
    }

    fn select_from_categories<S: Into<String>>(modifiers: Option<S>) -> String {
        let modifiers = modifiers
            .map_or(String::new(), S::into);

        return format!(r#"
            SELECT category_id, name, type, _origin, _creation_timestamp, _change_timestamp, _removal_timestamp
              FROM categories
                {}
        "#, modifiers);
    }

    fn select_from_plans<S: Into<String>>(modifiers: Option<S>) -> String {
        let modifiers = modifiers
            .map_or(String::new(), S::into);

        return format!(r#"
            SELECT plan_id, category_id, name, amount_limit, _origin, _creation_timestamp, _change_timestamp, _removal_timestamp
              FROM plans
                {}
        "#, modifiers);
    }
}


impl DbStorage {
    fn category_from_row(row: &rusqlite::Row<'_>) -> Result<EncryptedCategory> {
        let meta_info = MetaInfo {
            origin: row.get(3)?,
            added_timestamp: row.get(4)?,
            changed_timestamp: row.get(5)?,
            removed_timestamp: row.get(6)?
        };

        Ok(EncryptedCategory { 
            id: row.get(0)?, 
            name: row.get(1)?, 
            category_type: row.get(2)?,
            meta_info: meta_info
        })
    }

    fn account_from_row(row: &rusqlite::Row<'_>) -> Result<EncryptedAccount> {
        let meta_info = MetaInfo {
            origin: row.get(4)?,
            added_timestamp: row.get(5)?,
            changed_timestamp: row.get(6)?,
            removed_timestamp: row.get(7)?
        };

        Ok(EncryptedAccount { 
            id: row.get(0)?, 
            name: row.get(1)?, 
            balance: row.get(2)?,
            initial_balance: row.get(3)?,
            meta_info: meta_info
        })
    }

    fn transaction_from_row(row: &rusqlite::Row<'_>) -> Result<EncryptedTransaction> {
        let meta_info = MetaInfo {
            origin: row.get(6)?,
            added_timestamp: row.get(7)?,
            changed_timestamp: row.get(8)?,
            removed_timestamp: row.get(9)?
        };

        Ok(EncryptedTransaction { 
            id: row.get(0)?, 
            timestamp: row.get(1)?, 
            description: row.get(2)?, 
            account_id: row.get(3)?, 
            category_id: row.get(4)?, 
            amount: row.get(5)?,
            meta_info: meta_info
        })
    }

    fn plan_from_row(row: &rusqlite::Row<'_>) -> Result<EncryptedPlan> {
        let meta_info = MetaInfo {
            origin: row.get(4)?,
            added_timestamp: row.get(5)?,
            changed_timestamp: row.get(6)?,
            removed_timestamp: row.get(7)?
        };

        Ok(EncryptedPlan {
            id: row.get(0)?,
            category_id: row.get(1)?,
            name: row.get(2)?,
            amount_limit: row.get(3)?,
            meta_info: meta_info
        })
    }
}
