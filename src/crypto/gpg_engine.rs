use std::ffi::CString;
use std::cell::{RefCell, RefMut};

use crate::error::{Error, Result};
use crate::location::Location;
use super::prng::Prng;
use super::engine::CryptoEngine;
use super::buffer::CryptoBuffer;
use super::symmetric::SymmetricCipher;
use super::key::{Key, KeyId, KeyHandle, KeyIdentifier};
use super::{MISSING_SECRET_KEY, KEY_IS_NOT_SUITABLE, ENCRYPTION_ERROR, DECRYPTION_ERROR, INVALID_ENGINE_STATE};


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


/// Encrypted passphrase holder.
struct EncryptedKey {
    /// Encrypted passphrase data. Initialized in constructor.
    encrypted_buffer: CryptoBuffer,

    /// Decrypted passphrase. Initialized once on demand.
    decrypted_buffer: CryptoBuffer,
}


impl EncryptedKey {
    /// Open and read encrypted passphrase.
    /// 
    /// * `path` - path to encrypted passphrase file
    pub fn new(path: &std::path::Path) -> Result<Self> {
        //
        // Just read encrypted content here and do nothing else
        //

        Ok(EncryptedKey { 
            encrypted_buffer: CryptoBuffer::from(std::fs::read(path)?), 
            decrypted_buffer: CryptoBuffer::default(), 
        })
    }

    /// Decrypt passphrase if not decrypted yet.
    /// 
    /// * `key` - key used to decrypt passphrase
    /// * `engine` - engine used to decrypt passphrase
    pub fn decrypt(&mut self, key: &<GpgCryptoEngine as CryptoEngine>::Key, engine: &GpgCryptoEngine) -> Result<()> {
        if self.decrypted_buffer.is_empty() {
            //
            // Decrypt key once and remember
            //

            self.decrypted_buffer = engine.decrypt_asymmetric(
                key, self.encrypted_buffer.as_bytes())?;
        }

        Ok(())
    }
}


/// GnuPG cryptographic engine.
/// 
/// This engine in fact uses GPG keys to wrap a symmetric key, that
/// is used to perform actual cryptographic transformations via 
/// [`CryptoEngine::encrypt`] and [`CryptoEngine::decrypt`] functions.
/// 
/// Asymmetric encryption functions are used to wrap the symmetric key.
/// This key is generated randomly at creation stage and saved in
/// encrypted form to file.
/// 
/// In another word, [`GpgCryptoEngine`] performs hybrid encryption.
pub struct GpgCryptoEngine {
    /// Internal engine handle.
    engine: gpgme::Gpgme,

    /// Internal context.
    ctx: RefCell<gpgme::Context>,

    /// Encrypted symmetric key provider.
    symmetric_key: Option<RefCell<EncryptedKey>>,
}


impl GpgCryptoEngine {
    /// Creates a cryptographic engine for information queries.
    /// This engine cannot be used for performing cryptographic operations.
    pub fn new_dummy() -> Result<Self> {
        Self::new()
    }

    /// Creates a cryptographic engine for bdgt and initializes it.
    pub fn create<L: Location>(loc: &L, key_id: &<Self as CryptoEngine>::KeyId) -> Result<Self> {
        //
        // Location for config may be absent
        //

        loc.create_if_absent()?;
        
        Self::new()
            .and_then(|engine| engine.create_symmetric_key(loc, key_id))
    }

    /// Opens a cryptographic engine for bdgt.
    pub fn open<L: Location>(loc: &L) -> Result<Self> {
        Self::new()
            .and_then(|engine| engine.open_symmetric_key(loc))
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
        let symmetric_key = self.decrypt_symmetric_key(key)?;

        let cipher = SymmetricCipher::new(symmetric_key.decrypted_buffer.as_bytes())?;
        cipher.encrypt(plaintext)
    }

    fn decrypt(&self, key: &Self::Key, ciphertext: &[u8]) -> Result<CryptoBuffer> {
        let symmetric_key = self.decrypt_symmetric_key(key)?;

        let cipher = SymmetricCipher::new(symmetric_key.decrypted_buffer.as_bytes())?;
        cipher.decrypt(ciphertext)
    }
}


impl GpgCryptoEngine {
    fn new() -> Result<Self> {
        let ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;

        Ok(GpgCryptoEngine { 
            engine: gpgme::init(),
            ctx: RefCell::new(ctx),
            symmetric_key: None,
        })
    }

    fn create_symmetric_key<L: Location>(self, loc: &L, key_id: &<Self as CryptoEngine>::KeyId) -> Result<Self> {
        //
        // Check if key exists and suitable for encryption
        //

        let key = self.lookup_key(key_id)?;

        //
        // Create a random key using standard PRNG (cryptographically secure)
        // and write it in encrypted form to file
        //

        let mut symmetric_key = CryptoBuffer::new_with_size(SymmetricCipher::key_size());
        Prng::new()
            .generate(symmetric_key.as_mut_bytes())?;

        let encrypted_key = self.encrypt_asymmetric(&key, symmetric_key.as_bytes())?;
        std::fs::write(Self::symmetric_key_file(loc), encrypted_key.as_bytes())?;

        //
        // Set passphrase file in engine just by common opening procedure
        //

        self.open_symmetric_key(loc)
    }

    fn open_symmetric_key<L: Location>(mut self, loc: &L) -> Result<Self> {
        let encrypted_symmetric_key = EncryptedKey::new(&Self::symmetric_key_file(loc))?;
        self.symmetric_key = Some(RefCell::new(encrypted_symmetric_key));

        Ok(self)
    }

    fn symmetric_key_file<L: Location>(loc: &L) -> std::path::PathBuf {
        loc.root()
            .join("symm")
    }
}


impl GpgCryptoEngine {
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

    fn decrypt_symmetric_key(&self, key: &<Self as CryptoEngine>::Key) -> Result<RefMut<'_, EncryptedKey>> {
        if self.symmetric_key.is_none() {
            return Err(Error::from_message(INVALID_ENGINE_STATE));
        }

        let mut borrowed_symmetric_key = self.symmetric_key
            .as_ref()
            .unwrap()
            .borrow_mut();

        borrowed_symmetric_key
            .decrypt(key, self)?;

        Ok(borrowed_symmetric_key)
    }

    fn encrypt_asymmetric(&self, key: &<Self as CryptoEngine>::Key, plaintext: &[u8]) -> Result<CryptoBuffer> {
        let keys = [key.native_handle()];
        let mut ciphertext = Vec::new();

        self.ctx
            .borrow_mut()
            .encrypt(keys, plaintext, &mut ciphertext)
            .map_err(Error::from)
            .and_then(Self::check_encryption_result)
            .map(|_| CryptoBuffer::from(ciphertext))
    }

    fn decrypt_asymmetric(&self, _key: &<Self as CryptoEngine>::Key, ciphertext: &[u8]) -> Result<CryptoBuffer> {
        let mut plaintext = Vec::new();

        self.ctx
            .borrow_mut()
            .decrypt(ciphertext, &mut plaintext)
            .map_err(Error::from)
            .and_then(Self::check_decryption_result)
            .map(|_| CryptoBuffer::from(plaintext))
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
