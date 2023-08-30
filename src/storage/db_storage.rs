use std::path;

use rusqlite;

use crate::error::{Result, Error};
use crate::location::Location;
use super::data::{EncryptedTransaction, EncryptedCategory, EncryptedAccount, Id};
use super::storage::DataStorage;


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
    fn add_transaction(&self, account: Id, transaction: EncryptedTransaction) -> Result<()> {
        Ok(())
    }

    fn remove_transaction(&self, transaction: Id) -> Result<()> {
        Ok(())
    }

    fn transactions_of(&self, account: Id) -> Result<Vec<EncryptedTransaction>> {
        Ok(Vec::new())
    }

    fn transactions_with(&self, category: Id) -> Result<Vec<EncryptedTransaction>> {
        Ok(Vec::new())
    }

    fn add_account(&self, account: EncryptedAccount) -> Result<()> {
        Ok(())
    }

    fn remove_account(&self, account: Id, force: bool) -> Result<()> {
        Ok(())
    }

    fn accounts(&self) -> Result<Vec<EncryptedAccount>> {
        Ok(Vec::new())
    }

    fn add_category(&self, category: EncryptedCategory) -> Result<()> {
        Ok(())
    }

    fn remove_category(&self, category: Id) -> Result<()> {
        Ok(())
    }

    fn categories(&self) -> Result<Vec<EncryptedCategory>> {
        Ok(Vec::new())
    }
}


impl DbStorage {
    fn create_db(&self) -> Result<()> {
        self.db
            .execute_batch(include_str!("../../sql/creation.sql"))
            .map_err(Error::from)
    }

    fn db_path<L: Location>(loc: &L) -> path::PathBuf {
        loc.root()
            .join("database")
    }
}
