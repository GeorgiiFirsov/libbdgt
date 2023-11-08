mod git_engine;
mod syncable;
mod engine;

pub use git_engine::GitSyncEngine;

pub(crate) use engine::SyncEngine;
pub(crate) use syncable::Syncable;


/// Error message for case of adding of new remote, 
/// when another one already exists.
const REMOTE_ALREADY_EXIST: &str = "Remote is already associated with repository";
