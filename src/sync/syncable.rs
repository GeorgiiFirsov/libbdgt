use crate::error::Result;


/// Trais that must be implemented by diff representation.
pub trait Diff {
    /// Write diff into a writer.
    /// 
    /// * `writer` - writer to write diff in
    fn write_into<W: std::io::Write>(&self, writer: &mut W) -> Result<()>;
}


/// Trait that defines synchronization interface.
pub trait Syncable {
    /// Type of diff. It must implement [`Diff`].
    type Diff : Diff;

    /// Create diff that represents changes since specified moment of time.
    /// 
    /// * `base` - moment to get diff since
    fn diff_since(&self, base: chrono::DateTime<chrono::Utc>) -> Result<Self::Diff>;

    /// Apply diffs one-by-one.
    /// 
    /// * `diffs` - container withs diffs to apple
    fn merge_diffs(&self, diffs: Vec<Self::Diff>) -> Result<()>;
}
