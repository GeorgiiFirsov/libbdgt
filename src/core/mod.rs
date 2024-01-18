mod budget;
mod config;
mod changelog;

pub use self::budget::Budget;
pub use self::config::{Config, InstanceId};

/// Error shown in case of malformed timestamp file.
const MALFORMED_TIMESTAMP: &str = "Timestamp file in repository is malformed";
