use chrono;


/// Identifier type.
pub type Id = usize;


/// Identifier for primary keys in structure.
/// 
/// [`Option`] is required because new instances don't have
/// an id at creation time.
pub type PrimaryId = Option<Id>;


/// Type of timestamps.
pub type Timestamp = chrono::DateTime<chrono::Utc>;


/// Types of categories.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CategoryType {
    /// Incomes
    Income,

    /// Spendings
    Outcome,
}


/// User-friendly transaction structure.
pub struct Transaction {
    /// Identifier
    pub id: PrimaryId,

    /// Creation time
    pub timestamp: Timestamp,

    /// Brief description
    pub description: String,

    /// Identifier of a category
    pub category_id: Id,

    /// Identifier of an account, which the transaction belongs to
    pub account_id: Id,

    /// Amount of money affected
    pub amount: isize,
}


/// Protected transaction structure.
/// 
/// For fields description refer to [`Transaction`].
#[derive(Clone)]
pub struct EncryptedTransaction {
    pub id: PrimaryId,
    pub timestamp: Timestamp,
    pub description: Vec<u8>,
    pub category_id: Id,
    pub account_id: Id,
    pub amount: Vec<u8>,
}


/// User-friendly category structure.
pub struct Category {
    /// Identifier
    pub id: PrimaryId,

    /// Name of the category
    pub name: String,

    /// Type of category
    pub category_type: CategoryType,
}


/// Protected category structure.
/// 
/// For fields description refer to [`Category`].
#[derive(Clone)]
pub struct EncryptedCategory {
    pub id: PrimaryId,
    pub name: Vec<u8>,
    pub category_type: CategoryType,
}


/// User-friendly account structure.
pub struct Account {
    /// Identifier
    pub id: PrimaryId,

    /// User-friendly account name
    pub name: String,

    /// Current account balance
    pub balance: isize,
}


/// Protected account structure.
/// 
/// For fields description refer to [`Account`].
#[derive(Clone)]
pub struct EncryptedAccount {
    pub id: PrimaryId,
    pub name: Vec<u8>,
    pub balance: Vec<u8>
}
