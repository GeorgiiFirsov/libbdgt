use crate::error::Result;
use crate::datetime::Timestamp;


/// Trait that defines synchronization interface.
pub trait Syncable {
    /// Type of serialization context.
    type Context;

    /// Merges remote changelog and exports the local one.
    ///
    /// * `timestamp_rw` - last synchronization time (the function overwrites
    ///                    this value after performing synchronization)
    /// * `last_instance_rw` - last synchronized instance identifier (the function
    ///                        overwrites this value after preforming synchronization)
    /// * `changelog_rw` - full changelog to merge (the function appends local changelog
    ///                    to this value after preforming synchronization)
    /// * `last_sync` - last synchronization timestamp
    /// * `context` - user-provided context
    fn merge_and_export_changes<Ts, Li, Cl>(&self, timestamp_rw: &mut Ts, last_instance_rw: &mut Li,
        changelog_rw: &mut Cl, last_sync: &Timestamp, context: &Self::Context) -> Result<()>
    where
        Ts: std::io::Read + std::io::Write,
        Li: std::io::Read + std::io::Write,
        Cl: std::io::Read + std::io::Write;
}
