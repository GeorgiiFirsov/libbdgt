use std::path;
use std::fs;

use dirs;

use super::location::Location;


/// App's location based on current user's home directory.
pub struct HomeLocation;


impl HomeLocation {
    /// Just creates an instance.
    pub fn new() -> Self {
        HomeLocation
    }
}


impl Location for HomeLocation {
    fn root(&self) -> path::PathBuf {
        dirs::home_dir()
            .unwrap()
            .join(".bdgt")
    }

    fn create_if_absent(&self) -> crate::error::Result<()> {
        let root = self.root();
        if !root.exists() {
            fs::create_dir_all(root)?;
        }

        Ok(())
    }
}
