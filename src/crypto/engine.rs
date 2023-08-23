use crate::error::Result;
use super::buffer::CryptoBuffer;


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

    /// Encrypts a BLOB using a public key.
    /// 
    /// * `key` - handle to a public key, taht is intended to be used for encryption
    /// * `plaintext` - data to encrypt
    fn encrypt(&mut self, key: &Self::Key, plaintext: &[u8]) -> Result<CryptoBuffer>;

    /// Decrypts a BLOB using a private key.
    /// 
    /// * `key` - handle to a private key, taht is intended to be used for decryption
    /// * `plaintext` - data to decrypt
    fn decrypt(&mut self, key: &Self::Key, ciphertext: &[u8]) -> Result<CryptoBuffer>;
}
