use crate::error::Result;
use super::buffer::CryptoBuffer;


/// KDF implementation struct.
pub(crate) struct Kdf;


impl Kdf {
    /// Derives a symmetric key from password using Scrypt algorithm.
    /// 
    /// * `pass` - password to derive key from
    /// * `salt` - salt to use for key derivation
    /// * `key_size` - size of key to derive in bytes
    pub(crate) fn derive_key(pass: &[u8], salt: &[u8], key_size: usize) -> Result<CryptoBuffer> {
        let mut result = CryptoBuffer::new_with_size(key_size);
        scrypt::scrypt(pass, salt, &scrypt::Params::recommended(), 
            result.as_mut_bytes())?;

        Ok(result)
    }
}