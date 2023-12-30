mod data;
mod storage;
mod db_storage;
mod remote_storage;
mod remote_db_storage;

pub use self::storage::DataStorage;
pub use self::db_storage::DbStorage;
pub use self::data::*;


/// Error message for DB consistency violation.
const CONSISTENCY_VIOLATION: &str = "Cannot remove item from DB because of another items referencing it";

/// Error message for removing of predefined item prohibition.
const CANNOT_DELETE_PREDEFINED: &str = "Cannot remove predefined item";

/// Error message for modifying of predefined item prohibition.
const CANNOT_MODIFY_PREDEFINED: &str = "Cannot modify predefined item";
