use crate::error::Result;
use super::syncable::Syncable;


/// Synchronization engine.
pub trait SyncEngine {
    /// Perform synchronization.
    /// 
    /// Receives remote updates, sends local updates and applies remote ones.
    /// 
    /// * `current_instance` - name of current app instance
    /// * `syncable` - object to perform syncronization for
    fn perform_sync<S: Syncable>(&self, current_instance: &str, syncable: &S) -> Result<()>;

    /// Add a remote to list.
    /// 
    /// * `remote` - url or another remote identifier
    fn add_remote(&self, remote: &str) -> Result<()>;
}