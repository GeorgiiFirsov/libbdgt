mod git_engine;
mod syncable;
mod engine;

pub use self::git_engine::GitSyncEngine;

pub(crate) use self::engine::SyncEngine;
pub(crate) use self::syncable::Syncable;


/// Error message for case of adding of new remote, 
/// when another one already exists.
const REMOTE_ALREADY_EXIST: &str = "Remote is already associated with repository";

/// Error shown in case of malformed timestamp file.
const MALFORMED_LAST_SYNC_TIMESTAMP: &str = "Last synchronization timestamp file is malformed";
