use chrono;


/// Identifier type.
pub type Id = usize;


/// Types of categories.
#[derive(Clone, Copy, PartialEq)]
pub enum CategoryType {
    /// Incomes
    Income,

    /// Spendings
    Outcome,
}


/// User-friendly transaction structure.
pub struct Transaction {
    /// Creation time
    pub datetime: chrono::DateTime<chrono::Utc>,

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
pub struct EncryptedTransaction {
    pub datetime: chrono::DateTime<chrono::Utc>,
    pub description: Vec<u8>,
    pub category_id: Id,
    pub account_id: Id,
    pub amount: Vec<u8>,
}


/// User-friendly category structure.
pub struct Category {
    /// Name of the category
    pub name: String,

    /// Type of category
    pub category_type: CategoryType,
}


/// Protected category structure.
/// 
/// For fields description refer to [`Category`].
pub struct EncryptedCategory {
    pub name: Vec<u8>,
    pub category_type: CategoryType,
}


/// User-friendly account structure.
pub struct Account {
    /// User-friendly account name
    pub name: String,

    /// Current account balance
    pub balance: isize,
}


/// Protected account structure.
/// 
/// For fields description refer to [`Account`].
pub struct EncryptedAccount {
    pub name: Vec<u8>,
    pub balance: Vec<u8>
}
