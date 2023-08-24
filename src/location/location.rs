use std::path;


/// Traits, that manages application's data location.
pub trait Location {
    /// Get root path of app's data location.
    fn root(&self) -> path::PathBuf;
}
