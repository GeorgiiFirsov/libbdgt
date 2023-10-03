use typenum::Unsigned;
use aes_gcm::aead::Aead;
use aes_gcm::{KeySizeUser, AeadCore, KeyInit};

use crate::error::{Result, Error};
use super::prng::Prng;
use super::buffer::CryptoBuffer;
use super::INVALID_SYMMETRIC_KEY;


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
pub(crate) struct SymmetricCipher {
    /// Internal cipher implementation.
    cipher: Cipher,
}


impl SymmetricCipher {
    /// Create a new cipher instance using specific key.
    /// 
    /// Key MUST have size equal to the cipher's required key size.
    /// 
    /// * `key` - key used to encrypt or decrypt data
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != Self::key_size() {
            return Err(Error::from_message(INVALID_SYMMETRIC_KEY));
        }

        Ok(SymmetricCipher { 
            cipher: Cipher::new(&Key::from_slice(key)) 
        })
    }

    /// Obtain key size in bytes.
    pub fn key_size() -> usize {
        Cipher::key_size()
    }

    /// Encrypt a BLOB.
    /// 
    /// * `plaintext` - data to encrypt.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<CryptoBuffer> {
        let nonce = Cipher::generate_nonce(Prng::new());

        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext)?;
        
        Ok(
            CryptoBuffer::from(nonce.as_slice())
                .append(ciphertext)
        )
    }

    /// Decrypt a BLOB.
    /// 
    /// * `ciphertext` - data to decrypt.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<CryptoBuffer> {
        let (nonce, ciphertext) = ciphertext.split_at(NonceSize::USIZE);
        let nonce = Nonce::from_slice(nonce);

        let plaintext = self.cipher
            .decrypt(&nonce, ciphertext)?;
        
        Ok(CryptoBuffer::from(plaintext))
    }
}
