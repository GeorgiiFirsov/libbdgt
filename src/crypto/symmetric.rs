use typenum::Unsigned;
use aes_gcm::aead::Aead;
use aes_gcm::{KeySizeUser, AeadCore, KeyInit};

use crate::error::Result;
use super::prng::create_prng;
use super::buffer::CryptoBuffer;


/// Actual internal cipher implementation.
/// For now `bdgt` uses AES-256 block cipher
/// in GCM mode.
/// 
/// Nonce has length of 96 bits for the cipher.
/// It seems to be secure, because non-negligible
/// probability of repeating appears after
/// generating 2 ^ 48 nonces, i.e. more than
/// 280 billion nonces can be generated.
type Cipher = aes_gcm::Aes256Gcm;


/// Type of key buffer for symmetric cipher.
type Key = aes_gcm::Key<Cipher>;


/// Type that represents a size of nonce.
type NonceSize = <Cipher as AeadCore>::NonceSize;


/// Type of nonce.
type Nonce = aes_gcm::Nonce<NonceSize>;


/// Symmetric cipher interface.
pub(crate) struct SymmetricCipher;


impl SymmetricCipher {
    /// Obtain key size in bytes.
    pub fn key_size() -> usize {
        Cipher::key_size()
    }

    /// Encrypt a BLOB.
    /// 
    /// * `key` - key used to encrypt data.
    /// * `plaintext` - data to encrypt.
    pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<CryptoBuffer> {
        let nonce = Cipher::generate_nonce(create_prng());

        let key = Key::from_slice(key);
        let cipher = Cipher::new(&key);

        let ciphertext = cipher.encrypt(&nonce, plaintext)?;
        
        Ok(
            CryptoBuffer::from(nonce.as_slice())
                .append(ciphertext)
        )
    }

    /// Decrypt a BLOB.
    /// 
    /// * `key` - key used to decrypt data.
    /// * `ciphertext` - data to decrypt.
    pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<CryptoBuffer> {
        let (nonce, ciphertext) = ciphertext.split_at(NonceSize::USIZE);
        let nonce = Nonce::from_slice(nonce);

        let key = Key::from_slice(key);
        let cipher = Cipher::new(&key);

        let plaintext = cipher.decrypt(&nonce, ciphertext)?;
        
        Ok(CryptoBuffer::from(plaintext))
    }
}
