use crate::error::Result;
use crate::location::Location;
use crate::crypto::{KeyIdentifier, CryptoEngine};


/// File with key identifier name.
const KEY_IDENTIFIER_FILE: &str = "key";

/// File with instance identifier name.
const INSTANCE_IDENTIFIER_FILE: &str = "instance";


/// Type of local bdgt instance identifier.
pub type InstanceId = String;


/// App's instance configuration, contains long-term info.
pub struct Config<Ce>
where
    Ce: CryptoEngine
{
    /// Identifier of a key used to encrypt and decrypt sensitive data.
    /// Id is represented in a native format for concrete cryptographic engine.
    key_id: Ce::KeyId,

    /// Identifier of a local bdgt instance.
    instance_id: InstanceId,
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
        let raw_id = std::fs::read_to_string(Self::key_file(loc))?;
        let instance_id = std::fs::read_to_string(Self::instance_file(loc))?;

        Ok(Config { 
            key_id: Ce::KeyId::from_str(raw_id.as_str()),
            instance_id: instance_id
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
        // Save key into a file, generate new instance identifier,
        // and then just open config :)
        //

        std::fs::write(Self::key_file(loc), 
            key_id.as_string())?;

        std::fs::write(Self::instance_file(loc), 
            Self::new_instance())?;

        Self::open(loc)
    }

    /// Obtain the stored key identifier.
    pub fn key_id(&self) -> &Ce::KeyId {
        &self.key_id
    }

    /// Obtain the stored instance identifier.
    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }
}


impl<Ce> Config<Ce>
where
    Ce: CryptoEngine,
    Ce::KeyId: KeyIdentifier
{
    fn key_file<L: Location>(loc: &L) -> std::path::PathBuf {
        loc.root()
            .join(KEY_IDENTIFIER_FILE)
    }

    fn instance_file<L: Location>(loc: &L) -> std::path::PathBuf {
        loc.root()
            .join(INSTANCE_IDENTIFIER_FILE)
    }
}


impl<Ce> Config<Ce>
where
    Ce: CryptoEngine,
    Ce::KeyId: KeyIdentifier
{
    fn new_instance() -> InstanceId {
        let mut buffer = uuid::Uuid::encode_buffer();
        uuid::Uuid::new_v4()
            .hyphenated()
            .encode_lower(&mut buffer)
            .to_owned()
    }
}
