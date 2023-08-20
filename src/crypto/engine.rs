use gpgme;

use crate::error::{Result};
use super::key::{Key, KeyId, ExportedKey};


/// Cryptographic engine wrapper.
pub struct CryptoEngine {
    /// Internal OpenPGP engine (GPG)
    engine: gpgme::Gpgme,

    /// Internal context
    ctx: gpgme::Context
}


impl CryptoEngine {
    /// Creates and initializes cryptographic engine for bdgt.
    pub fn create() -> Result<CryptoEngine> {
        let ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;

        Ok(
            CryptoEngine { 
                engine: gpgme::init(),
                ctx: ctx
            }
        )
    }

    /// Looks for a key with specific identifier.
    /// 
    /// * `id` - identifier of a key to look for
    pub fn lookup_key(&mut self, id: &KeyId) -> Result<Key> {
        let internal_key = self.ctx
            .get_key(id.native_id())?;

        Ok(Key::new(internal_key, id))
    }

    /// Exports a public key.
    /// 
    /// * `key` - key handle
    pub fn export_key(&mut self, key: &Key) -> Result<ExportedKey> {
        self.internal_export_key(key, gpgme::ExportMode::MINIMAL)
    }

    /// Exports a private key.
    /// 
    /// * `key` - key handle
    pub fn export_secret_key(&mut self, key: &Key) -> Result<ExportedKey> {
        self.internal_export_key(key, gpgme::ExportMode::SECRET)
    }

    fn internal_export_key(&mut self, key: &Key, mode: gpgme::ExportMode) -> Result<ExportedKey> {
        //
        // GPG backend works only with iterables, hence I create an array with one single element
        //
        let keys = [key.native_handle()];

        let mut out = Vec::new();
        self.ctx
            .export_keys(keys, mode, &mut out)?;

        Ok(ExportedKey::new(out))
    } 
}
