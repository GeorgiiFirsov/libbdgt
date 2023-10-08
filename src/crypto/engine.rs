use crate::error::Result;
use super::key::KeyIdentifier;
use super::buffer::CryptoBuffer;


/// Cryptographic engine trait. 
/// 
/// This trait is very generic. It does not specify, how
/// encryption is performed, i.e. encryption can be symmetric,
/// asymmetric or hybrid. Furthermore, engine can support
/// different encryption types depending on key type.
pub trait CryptoEngine {
    /// Key identifier wrapper type, that hides engine-specific stuff behind.
    type KeyId : KeyIdentifier;

    /// Key wrapper type, that hides engine-specific stuff behind.
    type Key;

    /// Returns a name of cryptographic engine.
    fn engine(&self) -> &'static str;

    /// Returns a version of cryptographic engine.
    fn version(&self) -> &'static str;

    /// Looks for a key with specific identifier in engine's key storage.
    /// 
    /// Key is returned if and only if it exists and is suitable for bdgt.
    /// 
    /// * `id` - identifier of a key to look for
    fn lookup_key(&self, id: &Self::KeyId) -> Result<Self::Key>;

    /// Encrypts a BLOB using a provided key.
    /// 
    /// This method is generic. It is not specified, which encryption 
    /// algorithm is used. It can be asymmetric, symmetric or hybrid
    /// encryption.
    /// 
    /// * `key` - handle to a key.
    /// * `plaintext` - data to encrypt
    fn encrypt(&self, key: &Self::Key, plaintext: &[u8]) -> Result<CryptoBuffer>;

    /// Decrypts a BLOB using a provided key.
    /// 
    /// This method is generic. It is not specified, which encryption 
    /// algorithm is used. It can be asymmetric, symmetric or hybrid
    /// encryption.
    /// 
    /// * `key` - handle to a key.
    /// * `ciphertext` - data to decrypt
    fn decrypt(&self, key: &Self::Key, ciphertext: &[u8]) -> Result<CryptoBuffer>;

    /// Encrypts a BLOB symmetrically using a provided key.
    /// 
    /// This method mey be unsupported by some engines.
    /// 
    /// * `key` - binary key.
    /// * `plaintext` - data to encrypt
    fn encrypt_symmetric(&self, key: &[u8], plaintext: &[u8]) -> Result<CryptoBuffer>;

    /// Decrypts a BLOB symmetrically using a provided key.
    /// 
    /// This method mey be unsupported by some engines.
    /// 
    /// * `key` - binary key.
    /// * `ciphertext` - data to decrypt
    fn decrypt_symmetric(&self, key: &[u8], ciphertext: &[u8]) -> Result<CryptoBuffer>;
}
