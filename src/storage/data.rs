use chrono;


/// Identifier type.
pub type Id = usize;


/// Types of categories.
pub enum CategoryType {
    /// Incomes
    Income,

    /// Spendings
    Outcome,

    /// Transfers
    Transfer,
}


/// User-friendly transaction structure.
pub struct Transaction {
    /// Creation time
    datetime: chrono::DateTime<chrono::Utc>,

    /// Brief description
    description: String,

    /// Identifier of a category
    category_id: Id,

    /// Identifier of an account, which the transaction belongs to
    account_id: Id,

    /// Amount of money affected
    amount: isize,
}


/// Protected transaction structure.
/// 
/// For fields description refer to [`Transaction`].
pub struct EncryptedTransaction {
    datetime: chrono::DateTime<chrono::Utc>,
    description: Vec<u8>,
    category_id: Id,
    account_id: Id,
    amount: Vec<u8>,
}


/// User-friendly category structure.
pub struct Category {
    /// Name of the category
    name: String,

    /// Type of category
    category_type: CategoryType,
}


/// Protected category structure.
/// 
/// For fields description refer to [`Category`].
pub struct EncryptedCategory {
    name: Vec<u8>,
    category_type: CategoryType,
}


/// User-friendly account structure.
pub struct Account {
    /// User-friendly account name
    name: String,

    /// Current account balance
    balance: usize,
}


/// Protected account structure.
/// 
/// For fields description refer to [`Account`].
pub struct EncryptedAccount {
    name: Vec<u8>,
    balance: Vec<u8>
}
