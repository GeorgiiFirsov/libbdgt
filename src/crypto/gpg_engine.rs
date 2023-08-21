use std::ffi::CString;

use gpgme;

use super::engine::CryptoEngine;
use crate::error::{Error, Result};
use super::key::{Key, KeyId, ExportedKey, KeyHandle, KeyIdentifierImpl};
use super::{MISSING_SECRET_KEY, KEY_IS_NOT_SUITABLE};


/// Engine-specific key ahndle type.
type NativeHandle = gpgme::Key;

impl KeyHandle for NativeHandle {
    fn is_good(&self) -> bool {
        !self.is_bad()
    }

    fn can_encrypt(&self) -> bool {
        self.can_encrypt()
    }
}


/// Engine-specific key identifier type.
type NativeId = CString;

impl KeyIdentifierImpl for NativeId {
    fn create(id: &str) -> Self {
        NativeId::new(id).unwrap()   
    }
}


/// GPG cryptographic engine
pub struct GpgCryptoEngine {
    /// Internal engine handle
    engine: gpgme::Gpgme,

    /// Internal context
    ctx: gpgme::Context
}


impl GpgCryptoEngine {
    /// Creates and initializes cryptographic engine for bdgt.
    pub fn new() -> Result<Self> {
        let ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;

        Ok(
            GpgCryptoEngine { 
                engine: gpgme::init(),
                ctx: ctx
            }
        )
    }

    fn internal_export_key(&mut self, key: &<GpgCryptoEngine as CryptoEngine>::Key, mode: gpgme::ExportMode) -> Result<ExportedKey> {
        //
        // GPG backend works only with iterables, hence I create an array with one single element
        //
        let keys = [key.native_handle()];

        let mut out = Vec::new();
        self.ctx
            .export_keys(keys, mode, &mut out)?;

        Ok(ExportedKey::new(out))
    }

    fn verify_key(&mut self, key: <GpgCryptoEngine as CryptoEngine>::Key) -> Result<<GpgCryptoEngine as CryptoEngine>::Key> {
        let id = key
            .id()
            .clone();

        //
        // Check if there is corresponding private key
        //

        let key_ids = [id.native_id()];
        let secret_keys = self.ctx.find_secret_keys(key_ids)?;

        if 0 == secret_keys.count() {
            return Err(Error::from_message_with_extra(MISSING_SECRET_KEY, id.to_string()));
        }

        //
        // Now let's verify if all key properties are satisfied
        //

        key.is_suitable()
            .then_some(key)
            .ok_or(Error::from_message_with_extra(KEY_IS_NOT_SUITABLE, id.to_string()))
    }
}


impl CryptoEngine for GpgCryptoEngine {
    type Key = Key<NativeHandle, NativeId>;
    type KeyId = KeyId<NativeId>;

    fn engine(&self) -> &'static str {
        "GnuPG"
    }

    fn version(&self) -> &'static str {
        self.engine.version()
    }

    fn lookup_key(&mut self, id: &Self::KeyId) -> Result<Self::Key> {
        let internal_key = self.ctx
            .get_key(id.native_id())?;

        self.verify_key(Key::new(internal_key, id))
    }

    fn export_key(&mut self, key: &Self::Key) -> Result<ExportedKey> {
        self.internal_export_key(key, gpgme::ExportMode::MINIMAL)
    }

    fn export_secret_key(&mut self, key: &Self::Key) -> Result<ExportedKey> {
        self.internal_export_key(key, gpgme::ExportMode::SECRET)
    }
}
