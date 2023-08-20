use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};

use gpgme;


/// Engine-specific key identifier type.
pub(crate) type NativeId = CString;

/// Reference to engine-specific key identifier type.
pub(crate) type NativeIdView = CStr;

/// Engine-specific key ahndle type.
pub(crate) type NativeHandle = gpgme::Key;


/// Structure representing key identifier.
/// 
/// Hides backend-specific details from user.
#[derive(Clone)]
pub struct KeyId {
    id: NativeId
}


impl KeyId {
    /// Returns a backend-specific key identifier representation.
    pub(crate) fn native_id(&self) -> &NativeIdView {
        &self.id
    }
}


impl From<&str> for KeyId {
    fn from(value: &str) -> Self {
        KeyId { id: CString::new(value).unwrap() }
    }
}


impl Display for KeyId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.native_id())
    }
}


/// Structure, that wraps a key handle.
pub struct Key {
    /// Internal backend-specific key handle
    key: NativeHandle,

    /// Copy of key identifier
    id: KeyId
}


impl Key {
    /// Creates a key handle from native key handle and identifier.
    /// 
    /// * `key` - native key handle
    /// * `id` - key identifier
    pub(crate) fn new(key: NativeHandle, id: &KeyId) -> Self {
        Key { 
            key: key,
            id: id.clone()
        }
    }

    /// Returns a native key handle.
    pub(crate) fn native_handle(&self) -> &NativeHandle {
        &self.key
    }

    /// Checks if the key is suitable for bdgt.
    /// 
    /// Key MUST NOT be expired, revoked, disabled, 
    /// MUST contain a secret key and MUST be able 
    /// to perform encryption.
    pub(crate) fn is_suitable(&self) -> bool {
        let is_good = !self.key.is_bad();
        let can_encrypt = self.key.can_encrypt();
        let has_secret_key = self.key.has_secret();

        is_good && has_secret_key && can_encrypt
    }
}


/// Struct for wrapping an exported key.
/// 
/// Implements [`core::ops::Drop`] trait, that erases internal 
/// key at destruction time.
pub struct ExportedKey {
    /// Raw key bytes
    key: Vec<u8>
}


impl ExportedKey {
    /// Creates a key from vector by moving it into a new object.
    /// 
    /// * `key` - raw key bytes
    pub(crate) fn new(key: Vec<u8>) -> Self {
        ExportedKey { key: key }
    }

    /// Returns read-only raw bytes of the stored key.
    pub fn as_raw(&self) -> &[u8] {
        &self.key
    }
}


impl Drop for ExportedKey {
    fn drop(&mut self) {
        //
        // Just zero stored memory
        //
        for e in self.key.iter_mut() {
            *e = 0u8;
        }
    }
}
