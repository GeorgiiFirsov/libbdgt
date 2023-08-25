use std::path;

use crate::error::Result;


/// Traits, that manages application's data location.
pub trait Location {
    /// Get root path of app's data location.
    fn root(&self) -> path::PathBuf;

    /// Create root directory if it doesn't exist.
    fn create_if_absent(&self) -> Result<()>;
}
