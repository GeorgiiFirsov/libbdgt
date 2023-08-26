use crate::crypto::{CryptoEngine, KeyIdentifier};
use super::storage::DataStorage;
use crate::config::Config;
use crate::error::Result;


/// Budget manager.
pub struct Budget<Ce, St>
where
    Ce: CryptoEngine,
    St: DataStorage
{
    /// Cryptographic engine used to encrypt sensitive data.
    crypto_engine: Ce,

    /// Storage used to store the data.
    storage: St,

    /// Key used to encrypt and decrypt sensitive data.
    key: Ce::Key
}


impl<Ce, St> Budget<Ce, St>
where
    Ce: CryptoEngine,
    St: DataStorage,
    Ce::KeyId: KeyIdentifier
{
    /// Creates a budget manager instance.
    /// 
    /// * `crypto_engine` - cryptographic engine used to encrypt sensitive data
    /// * `storage` - storage used to store data
    /// * `config` - app's configuration
    pub fn new(mut crypto_engine: Ce, storage: St, config: Config<Ce>) -> Result<Self>
    {
        let key = crypto_engine
            .lookup_key(config.key_id())?;

        Ok(Budget { 
            crypto_engine: crypto_engine, 
            storage: storage,
            key: key
        })
    }

    /// Underlying cryptographic engine name.
    pub fn engine(&self) -> &str {
        self.crypto_engine.engine()
    }

    /// Underlying cryptofgraphic engine version.
    pub fn engine_version(&self) -> &str {
        self.crypto_engine.version()
    }
}