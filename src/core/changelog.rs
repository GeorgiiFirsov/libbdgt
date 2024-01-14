use crate::storage::{Transaction, Account, Category, Plan};


/// Simple changelog representation for some items.
pub(crate) struct SimpleChangelog<T> {
    /// Added items.
    pub added: Vec<T>,

    /// Changed items.
    pub changed: Vec<T>,

    /// Removed items.
    pub removed: Vec<T>,
}


/// Database changelog representation.
pub(crate) struct Changelog {
    /// Diff for accounts.
    pub accounts: SimpleChangelog<Account>,

    /// Diff for categories.
    pub categories: SimpleChangelog<Category>,

    /// Diff for transactions.
    pub transactions: SimpleChangelog<Transaction>,

    /// Diff for plans.
    pub plans: SimpleChangelog<Plan>,
}
