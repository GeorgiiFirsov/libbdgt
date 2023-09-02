use std::path;

use rusqlite;

use crate::error::{Result, Error};
use crate::location::Location;
use super::data::{EncryptedTransaction, EncryptedCategory, EncryptedAccount, Id, CategoryType};
use super::storage::DataStorage;


/// Implementation of [`rusqlite::ToSql`] trait for [`CategoryType`].
/// 
/// [`CategoryType::Income`] translates into 0, [`CategoryType::Outcome`] -- into 1.
impl rusqlite::ToSql for CategoryType {
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
    /// Opens an existing database in provided location.
    /// 
    /// * `loc` - storage location provider
    pub fn open<L: Location>(loc: &L) -> Result<Self> {
        Ok(DbStorage { 
            db: rusqlite::Connection::open(Self::db_path(loc))?
        })
    }

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
}


impl DataStorage for DbStorage {
    fn add_transaction(&self, transaction: EncryptedTransaction) -> Result<()> {
        let statement_fmt = r#"
            INSERT INTO transactions (timestamp, description, account_id, category_id, amount)
            VALUES (?1, ?2, ?3, ?4, ?5)
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![transaction.datetime, transaction.description, 
                transaction.account_id, transaction.category_id, transaction.amount])?;
        
        Ok(())
    }

    fn remove_transaction(&self, transaction: Id) -> Result<()> {
        let statement_fmt = r#"
            DELETE FROM transactions
             WHERE transaction_id = ?1
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![transaction])?;

        Ok(())
    }

    fn transactions(&self) -> Result<Vec<EncryptedTransaction>> {
        let statement = r#"
            SELECT transaction_id, timestamp, description, account_id, category_id, amount
              FROM transactions
        "#;

        self.query(statement, Self::transaction_from_row)
    }

    fn add_account(&self, account: EncryptedAccount) -> Result<()> {
        let statement_fmt = r#"
            INSERT INTO accounts (name, balance)
            VALUES (?1, ?2)
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![account.name, account.balance])?;

        Ok(())
    }

    fn remove_account(&self, account: Id, force: bool) -> Result<()> {
        if force {
            //
            // Forced removal is requested, hence I need to remove
            // all transactions first
            //

            let statement_fmt = r#"
                DELETE FROM transactions
                 WHERE account_id = ?1
            "#;

            self.db
                .execute(statement_fmt, rusqlite::params![account])?;
        }

        let statement_fmt = r#"
            DELETE FROM accounts
             WHERE account_id = ?1
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![account])?;

        Ok(())
    }

    fn accounts(&self) -> Result<Vec<EncryptedAccount>> {
        let statement = r#"
            SELECT account_id, name, balance
              FROM accounts
        "#;

        self.query(statement, Self::account_from_row)
    }

    fn add_category(&self, category: EncryptedCategory) -> Result<()> {
        let statement_fmt = r#"
            INSERT INTO categories (name, type)
            VALUES (?1, ?2)
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![category.name, category.category_type])?;

        Ok(())
    }

    fn remove_category(&self, category: Id) -> Result<()> {
        let statement_fmt = r#"
            DELETE FROM categories
             WHERE category_id = ?1
        "#;

        self.db
            .execute(statement_fmt, rusqlite::params![category])?;

        Ok(())
    }

    fn categories(&self) -> Result<Vec<EncryptedCategory>> {
        let statement = r#"
            SELECT category_id, name, type 
              FROM categories
        "#;

        self.query(statement, Self::category_from_row)
    }
}


impl DbStorage {
    fn create_db(&self) -> Result<()> {
        let create_statement = r#"
            CREATE TABLE accounts (
                account_id          SERIAL      PRIMARY KEY,
                current_balance     BYTEA       NOT NULL,
                name                BYTEA       NOT NULL
            );
                
            CREATE TABLE categories (
                category_id         SERIAL      PRIMARY KEY,
                name                BYTEA       NOT NULL,
                type                TINYINT     NOT NULL
            );
                
            CREATE TABLE transactions (
                transaction_id      SERIAL      PRIMARY KEY,
                timestamp           DATETIME    NOT NULL,
                description         BYTEA       NOT NULL,    
                account_id          SERIAL      REFERENCES accounts(account_id),
                category_id         SERIAL      REFERENCES categories(category_id),
                amount              BYTEA       NOT NULL
            );
        "#;

        self.db
            .execute_batch(create_statement)
            .map_err(Error::from)
    }

    fn db_path<L: Location>(loc: &L) -> path::PathBuf {
        loc.root()
            .join("database")
    }

    fn query<T, P>(&self, statement: &str, parse: P) -> Result<Vec<T>>
    where
        P: Fn(&rusqlite::Row<'_>) -> Result<T>
    {
        let mut statement = self.db.prepare(statement)?;
        let mut rows = statement.query([])?;

        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(parse(row)?)
        }

        Ok(result)
    }
}


impl DbStorage {
    fn category_from_row(row: &rusqlite::Row<'_>) -> Result<EncryptedCategory> {
        Ok(EncryptedCategory { 
            id: row.get(0)?, 
            name: row.get(1)?, 
            category_type: row.get(2)?
        })
    }

    fn account_from_row(row: &rusqlite::Row<'_>) -> Result<EncryptedAccount> {
        Ok(EncryptedAccount { 
            id: row.get(0)?, 
            name: row.get(1)?, 
            balance: row.get(2)? 
        })
    }

    fn transaction_from_row(row: &rusqlite::Row<'_>) -> Result<EncryptedTransaction> {
        Ok(EncryptedTransaction { 
            id: row.get(0)?, 
            datetime: row.get(1)?, 
            description: row.get(2)?, 
            category_id: row.get(3)?, 
            account_id: row.get(4)?, 
            amount: row.get(5)? 
        })
    }
}
