use crate::error::Result;
use super::key::ExportedKey;


/// Cryptographic engine trait
pub trait CryptoEngine {
    /// Key identifier wrapper type, that hides engine-specific stuff behind.
    type KeyId;

    /// Key wrapper type, that hides engine-specific stuff behind.
    type Key;

    /// Returns a name of cryptographic engine.
    fn engine(&self) -> &'static str;

    /// Returns a version of cryptographic engine.
    fn version(&self) -> &'static str;

    /// Looks for a key with specific identifier.
    /// 
    /// Key is returned if and only if it exists and is suitable for bdgt.
    /// 
    /// * `id` - identifier of a key to look for
    fn lookup_key(&mut self, id: &Self::KeyId) -> Result<Self::Key>;

    /// Exports a public key.
    /// 
    /// * `key` - key handle
    fn export_key(&mut self, key: &Self::Key) -> Result<ExportedKey>;

    /// Exports a private key.
    /// 
    /// * `key` - key handle
    fn export_secret_key(&mut self, key: &Self::Key) -> Result<ExportedKey>;
}
