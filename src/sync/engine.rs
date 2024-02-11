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
    fn perform_sync<S: Syncable>(&self, current_instance: &S::InstanceId, syncable: &S, context: &S::Context) -> Result<()>;

    /// Add a remote. Note, that there can be only one remote. Therefore,
    /// the function fails, if there's already a remote associated.
    /// 
    /// * `remote` - url or another remote identifier
    fn add_remote(&self, remote: &str) -> Result<()>;

    /// Remove existing remote.
    fn remove_remote(&self) -> Result<()>;

    /// Changes existing remote.
    /// 
    /// * `remote` - url or another remote identifier
    fn change_remote(&self, remote: &str) -> Result<()>;
}
