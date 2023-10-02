use aes_gcm::{KeySizeUser, AeadCore, KeyInit};

use crate::error::Result;
use super::buffer::CryptoBuffer;


/// Type of key buffer for symmetric cipher.
pub(crate) type Key<'a> = &'a [u8];


/// Actual internal cipher implementation.
/// For now `bdgt` uses AES-256 block cipher
/// in GCM mode.
type Cipher = aes_gcm::Aes256Gcm;


/// Symmetric cipher interface.
pub(crate) struct SymmetricCipher;


impl SymmetricCipher {
    /// Obtain key size in bytes.
    pub fn key_size() -> usize {
        aes_gcm::Aes256Gcm::key_size()
    }

    /// Encrypt a BLOB.
    /// 
    /// * `key` - key used to encrypt data.
    /// * `plaintext` - data to encrypt.
    pub fn encrypt(_key: Key, plaintext: &[u8]) -> Result<CryptoBuffer> {
        // TODO
        Ok(CryptoBuffer::from(plaintext))
    }

    /// Decrypt a BLOB.
    /// 
    /// * `key` - key used to decrypt data.
    /// * `ciphertext` - data to decrypt.
    pub fn decrypt(_key: Key, ciphertext: &[u8]) -> Result<CryptoBuffer> {
        // TODO
        Ok(CryptoBuffer::from(ciphertext))
    }
}
