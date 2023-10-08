use crate::error::Result;
use super::location::Location;


/// Root folder for app's data.
const ROOT_FOLDER: &str = ".bdgt";


/// App's location based on current user's home directory.
pub struct HomeLocation;


impl HomeLocation {
    /// Just creates an instance.
    pub fn new() -> Self {
        HomeLocation
    }
}


impl Location for HomeLocation {
    fn root(&self) -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap()
            .join(ROOT_FOLDER)
    }

    fn exists(&self) -> bool {
        self.root()
            .exists()
    }

    fn create_if_absent(&self) -> Result<()> {
        if !self.exists() {
            std::fs::create_dir_all(self.root())?;
        }

        Ok(())
    }
}
