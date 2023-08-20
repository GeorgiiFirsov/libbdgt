use std::ffi::{CStr, CString};

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
