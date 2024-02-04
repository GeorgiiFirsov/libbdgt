use serde::{Serialize, Deserialize};

use crate::core::InstanceId;
use crate::datetime::Timestamp;


/// Identifier type.
pub type Id = [u8; 16];


/// Identifier for primary keys in structure.
/// 
/// [`Option`] is required because new instances don't have
/// an id at creation time.
pub type PrimaryId = Option<Id>;


/// Types of categories.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CategoryType {
    /// Incomes
    Income,

    /// Spendings
    Outcome,
}


/// Meta information about an entity
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct MetaInfo {
    // Origin (instance, where an object was created)
    pub origin: [u8; 16],

    // Creation timestamp
    pub added_timestamp: Option<Timestamp>,
    
    // Change timestamp
    pub changed_timestamp: Option<Timestamp>,

    // Removal timestamp
    pub removed_timestamp: Option<Timestamp>
}


impl MetaInfo {
    /// Constructs a meta info instance with given timestamps.
    /// 
    /// * `origin` - identifer of an instance, which item was created on
    /// * `added_timestamp` - creation timestamp or `None`
    /// * `changed_timestamp` - change timestamp or `None`
    /// * `removed_timestamp` - removal timestamp or `None`
    pub fn new(origin: &InstanceId, added_timestamp: Option<Timestamp>, 
        changed_timestamp: Option<Timestamp>, removed_timestamp: Option<Timestamp>) -> Self 
    {
        MetaInfo {
            origin: origin.into_bytes(),
            added_timestamp, 
            changed_timestamp, 
            removed_timestamp
        }
    }
}


/// User-friendly transaction structure.
#[derive(Serialize, Deserialize)]
pub struct Transaction {
    /// Identifier
    pub id: PrimaryId,

    /// Creation time
    pub timestamp: Timestamp,

    /// Brief description
    pub description: String,

    /// Identifier of an account, which the transaction belongs to
    pub account_id: Id,

    /// Identifier of a category
    pub category_id: Id,

    /// Amount of money affected
    pub amount: isize,

    /// Meta info
    pub meta_info: MetaInfo
}


/// Protected transaction structure.
/// 
/// For fields description refer to [`Transaction`].
#[derive(Clone)]
pub struct EncryptedTransaction {
    pub id: PrimaryId,
    pub timestamp: Timestamp,
    pub description: Vec<u8>,
    pub account_id: Id,
    pub category_id: Id,
    pub amount: Vec<u8>,
    pub meta_info: MetaInfo
}


/// User-friendly category structure.
#[derive(Serialize, Deserialize)]
pub struct Category {
    /// Identifier
    pub id: PrimaryId,

    /// Name of the category
    pub name: String,

    /// Type of category
    pub category_type: CategoryType,

    /// Meta info
    pub meta_info: MetaInfo
}


/// Protected category structure.
/// 
/// For fields description refer to [`Category`].
#[derive(Clone)]
pub struct EncryptedCategory {
    pub id: PrimaryId,
    pub name: Vec<u8>,
    pub category_type: CategoryType,
    pub meta_info: MetaInfo
}


/// User-friendly account structure.
#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    /// Identifier
    pub id: PrimaryId,

    /// User-friendly account name
    pub name: String,

    /// Current account balance
    pub balance: isize,

    /// Initial account balance
    pub initial_balance: isize,

    /// Meta info
    pub meta_info: MetaInfo
}


/// Protected account structure.
/// 
/// For fields description refer to [`Account`].
#[derive(Clone)]
pub struct EncryptedAccount {
    pub id: PrimaryId,
    pub name: Vec<u8>,
    pub balance: Vec<u8>,
    pub initial_balance: Vec<u8>,
    pub meta_info: MetaInfo
}


/// User-friendly plan structure.
#[derive(Serialize, Deserialize)]
pub struct Plan {
    /// Identifier
    pub id: PrimaryId,

    /// Identifier of corresponding category
    pub category_id: Id,

    /// User-friendly plan name
    pub name: String,

    /// Current plan balance
    pub amount_limit: isize,

    /// Meta info
    pub meta_info: MetaInfo
}


/// Protected plan structure.
/// 
/// For fields description refer to [`Plan`].
#[derive(Clone)]
pub struct EncryptedPlan {
    pub id: PrimaryId,
    pub category_id: Id,
    pub name: Vec<u8>,
    pub amount_limit: Vec<u8>,
    pub meta_info: MetaInfo
}
