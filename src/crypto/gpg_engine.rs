use std::ffi::CString;
use std::cell::RefCell;

use crate::error::{Error, Result};
use crate::location::Location;
use super::engine::CryptoEngine;
use super::buffer::CryptoBuffer;
use super::key::{Key, KeyId, KeyHandle, KeyIdentifier};
use super::{MISSING_SECRET_KEY, KEY_IS_NOT_SUITABLE, ENCRYPTION_ERROR, DECRYPTION_ERROR};


/// Engine-specific key identifier type.
type NativeId = CString;

impl KeyIdentifier for NativeId {
    fn from_str(id: &str) -> Self {
        NativeId::new(id).unwrap()   
    }

    fn as_string(&self) -> String {
        self.to_str()
            .unwrap()
            .to_owned()
    }
}


/// Engine-specific key handle type.
type NativeHandle = gpgme::Key;

impl KeyHandle for NativeHandle {
    fn is_good(&self) -> bool {
        !self.is_bad()
    }

    fn can_encrypt(&self) -> bool {
        self.can_encrypt()
    }
}


/// GPG cryptographic engine
pub struct GpgCryptoEngine {
    /// Internal engine handle
    engine: gpgme::Gpgme,

    /// Internal context
    ctx: RefCell<gpgme::Context>
}


impl GpgCryptoEngine {
    /// Creates a cryptographic engine for information queries.
    /// This engine cannot be used for performing cryptographic operations.
    pub fn new_dummy() -> Result<Self> {
        Self::new()
    }

    /// Creates a cryptographic engine for bdgt and initializes it.
    pub fn create<L: Location>(loc: &L, key: &<Self as CryptoEngine>::KeyId) -> Result<Self> {
        //
        // Location for config may be absent
        //

        loc.create_if_absent()?;
        
        Self::new()
            .and_then(|engine| engine.create_pwd(loc, key))
    }

    /// Opens a cryptographic engine for bdgt.
    pub fn open<L: Location>(loc: &L) -> Result<Self> {
        Self::new()
            .and_then(|engine| engine.open_pwd(loc))
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

    fn lookup_key(&self, id: &Self::KeyId) -> Result<Self::Key> {
        let internal_key = self.ctx
            .borrow_mut()
            .get_key(id.native_id())?;

        self.verify_key(Key::new(internal_key, id))
    }

    fn encrypt(&self, key: &Self::Key, plaintext: &[u8]) -> Result<CryptoBuffer> {
        let keys = [key.native_handle()];
        let mut ciphertext = Vec::new();

        self.ctx
            .borrow_mut()
            .encrypt(keys, plaintext, &mut ciphertext)
            .map_err(Error::from)
            .and_then(Self::check_encryption_result)
            .map(|_| CryptoBuffer::new(ciphertext))
    }

    fn decrypt(&self, _key: &Self::Key, ciphertext: &[u8]) -> Result<CryptoBuffer> {
        let mut plaintext = Vec::new();

        self.ctx
            .borrow_mut()
            .decrypt(ciphertext, &mut plaintext)
            .map_err(Error::from)
            .and_then(Self::check_decryption_result)
            .map(|_| CryptoBuffer::new(plaintext))
    }
}


impl GpgCryptoEngine {
    fn new() -> Result<Self> {
        let ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;

        Ok(GpgCryptoEngine { 
            engine: gpgme::init(),
            ctx: RefCell::new(ctx)
        })
    }

    fn create_pwd<L: Location>(mut self, _loc: &L, key: &<Self as CryptoEngine>::KeyId) -> Result<Self> {
        //
        // Check if key exists and suitable for encryption
        //

        let _key = self.lookup_key(key)?;
        Ok(self)
    }

    fn open_pwd<L: Location>(mut self, _loc: &L) -> Result<Self> {
        Ok(self)
    }

    fn verify_key(&self, key: <Self as CryptoEngine>::Key) -> Result<<Self as CryptoEngine>::Key> {
        //
        // Borrow context for the entire function life
        //

        let mut borrowed_ctx = self.ctx.borrow_mut();

        //
        // Check if there is corresponding private key
        //

        let id = key
            .id()
            .clone();

        let key_ids = [id.native_id()];
        let secret_keys = borrowed_ctx.find_secret_keys(key_ids)?;

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
