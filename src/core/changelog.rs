use serde::{Serialize, Deserialize};

use crate::error::{Result, Error};
use crate::storage::{Transaction, Account, Category, Plan};


/// Simple changelog representation for some items.
#[derive(Serialize, Deserialize)]
pub(crate) struct SimpleChangelog<T> {
    /// Added items.
    pub added: Vec<T>,

    /// Changed items.
    pub changed: Vec<T>,

    /// Removed items.
    pub removed: Vec<T>,
}


/// Database changelog representation.
#[derive(Serialize, Deserialize)]
pub(crate) struct Changelog {
    /// Accounts changelog.
    pub accounts: SimpleChangelog<Account>,

    /// Categories changelog.
    pub categories: SimpleChangelog<Category>,

    /// Transactions changelog.
    pub transactions: SimpleChangelog<Transaction>,

    /// Plans changelog.
    pub plans: SimpleChangelog<Plan>,
}


impl Changelog {
    /// Creates a new changelog object from binary representation.
    /// 
    /// * `binary_changelog` - binary changelog representation
    pub(crate) fn new(binary_changelog: &[u8]) -> Result<Self> {
        rmp_serde::from_read(binary_changelog)
            .map_err(Error::from)
    }

    /// Appends another changelog to the current one.
    /// 
    /// * `changelog` - a changelog to append
    pub(crate) fn append(&mut self, mut changelog: Changelog) -> Result<()> {
        self.accounts.added.append(&mut changelog.accounts.added);
        self.accounts.changed.append(&mut changelog.accounts.changed);
        self.accounts.removed.append(&mut changelog.accounts.removed);

        self.categories.added.append(&mut changelog.categories.added);
        self.categories.changed.append(&mut changelog.categories.changed);
        self.categories.removed.append(&mut changelog.categories.removed);

        self.transactions.added.append(&mut changelog.transactions.added);
        self.transactions.changed.append(&mut changelog.transactions.changed);
        self.transactions.removed.append(&mut changelog.transactions.removed);

        self.plans.added.append(&mut changelog.plans.added);
        self.plans.changed.append(&mut changelog.plans.changed);
        self.plans.removed.append(&mut changelog.plans.removed);

        Ok(())
    }

    /// Converts current changelog into a binary representation.
    pub(crate) fn as_bytes(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self)
            .map_err(Error::from)
    }
}
