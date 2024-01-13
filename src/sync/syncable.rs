use crate::error::Result;


/// Trait that defines synchronization interface.
pub trait Syncable {
    /// Type of serialization context.
    type Context;

    /// Merges remote changelog and exports the local one.
    /// 
    /// * `timestamp` - last synchronization time (the function overwrites 
    ///                 this value after performing synchronization)
    /// * `last_instance` - last synchronized instance identifier (the function
    ///                     overwrites this value after preforming synchronization)
    /// * `changelog` - full changelog to merge (the function appends local changelog
    ///                 to this value after preforming synchronization)
    /// * `context` - user-provided context
    fn merge_and_export_changes<Ts, Li, Cl>(&self, timestamp: &mut Ts, last_instance: &mut Li, changelog: &mut Cl, context: &Self::Context) -> Result<()>
    where
        Ts: std::io::Read + std::io::Write,
        Li: std::io::Read + std::io::Write,
        Cl: std::io::Read + std::io::Write;
}
