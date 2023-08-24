use std::ffi::CString;

use gpgme;

use super::engine::CryptoEngine;
use super::buffer::CryptoBuffer;
use crate::error::{Error, Result};
use super::key::{Key, KeyId, KeyHandle, KeyIdentifierImpl};
use super::{MISSING_SECRET_KEY, KEY_IS_NOT_SUITABLE, ENCRYPTION_ERROR, DECRYPTION_ERROR};


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

    fn str(&self) -> String {
        self.to_str()
            .unwrap()
            .to_owned()
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

    fn check_encryption_result(result: gpgme::EncryptionResult) -> Result<()> {
        let invalid_count = result
            .invalid_recipients()
            .count();

        (0 == invalid_count)
            .then_some(())
            .ok_or(Error::from_message(ENCRYPTION_ERROR))
    }

    fn check_decryption_result(result: gpgme::DecryptionResult) -> Result<()> {
        let correct = !result.is_wrong_key_usage();

        correct
            .then_some(())
            .ok_or(Error::from_message(DECRYPTION_ERROR))
    }
}


impl CryptoEngine for GpgCryptoEngine {
    type Key = Key<NativeHandle, NativeId>;
    type KeyId = KeyId<NativeId>;

    fn engine(&self) -> &'static str {
        "GnuPG"
    }

    fn version(&self) -> &'static str {
        self.engine
            .version()
    }

    fn lookup_key(&mut self, id: &Self::KeyId) -> Result<Self::Key> {
        let internal_key = self.ctx
            .get_key(id.native_id())?;

        self.verify_key(Key::new(internal_key, id))
    }

    fn encrypt(&mut self, key: &Self::Key, plaintext: &[u8]) -> Result<CryptoBuffer> {
        let keys = [key.native_handle()];
        let mut ciphertext = Vec::new();

        self.ctx
            .encrypt(keys, plaintext, &mut ciphertext)
            .map_err(Error::from)
            .and_then(Self::check_encryption_result)
            .map(|_| CryptoBuffer::new(ciphertext))
    }

    fn decrypt(&mut self, _key: &Self::Key, ciphertext: &[u8]) -> Result<CryptoBuffer> {
        let mut plaintext = Vec::new();

        self.ctx
            .decrypt(ciphertext, &mut plaintext)
            .map_err(Error::from)
            .and_then(Self::check_decryption_result)
            .map(|_| CryptoBuffer::new(plaintext))
    }
}
