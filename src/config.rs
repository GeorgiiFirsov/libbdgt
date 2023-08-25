use std::fs;
use std::path;

use crate::error::Result;
use crate::location::Location;
use crate::crypto::{KeyIdentifier, CryptoEngine};


/// App's configuration, contains long-term info.
pub struct Config<Ce>
where
    Ce: CryptoEngine
{
    /// Identifier of a key used to encrypt and decrypt sensitive data.
    /// Id is represented in a native format for concrete cryptographic engine.
    key_id: Ce::KeyId,
}


impl<Ce> Config<Ce>
where
    Ce: CryptoEngine,
    Ce::KeyId: KeyIdentifier
{
    /// Opens an existing storage and load stored configuration.
    /// 
    /// * `loc` - storage location provider
    pub fn open<L: Location>(loc: &L) -> Result<Self> {
        let raw_id = fs::read_to_string(Self::key_file(loc))?;

        Ok(Config { 
            key_id: Ce::KeyId::from_str(raw_id.as_str())
        })
    }

    /// Creates a new storage and then loads configuration.
    /// 
    /// * `loc` - storage location provider
    /// * `key_id` - key identifier
    pub fn create<L: Location>(loc: &L, key_id: &Ce::KeyId) -> Result<Self> {
        //
        // Check is root location exists and create it if necessary
        //

        loc.create_if_absent()?;

        //
        // Save key into a file and then just open config :)
        //

        fs::write(Self::key_file(loc), 
            key_id.as_string())?;

        Self::open(loc)
    }

    /// Obtain the stored key identifier.
    pub fn key_id(&self) -> &Ce::KeyId {
        &self.key_id
    }

    fn key_file<L: Location>(loc: &L) -> path::PathBuf {
        loc.root()
            .join("key")
    }
}