use std::fmt::{Display, Formatter, Debug};

use trait_set::trait_set;


/// Key handle trait.
/// 
/// Implemented for concrete engine-specific handles.
pub trait KeyHandle {
    /// Checks if key is not expired, not revoked and not disabled.
    fn is_good(&self) -> bool;

    /// Checks if key is suitable for encryption.
    fn can_encrypt(&self) -> bool;
}


/// Internal key identifier trait.
/// 
/// Implemented for concrete engine-specific identifier types.
pub trait KeyIdentifierImpl {
    /// Creates an identifier from string reference.
    /// 
    /// * `id` - identifier as string
    fn create(id: &str) -> Self;

    /// Converts identifier into a string.
    fn str(&self) -> String;
}


// Trait aliases are defined using `trait-set` crate,
// because complex trait aliases are unstable for now
trait_set! {
    /// Key identifier trait.
    pub trait KeyIdentifier = Clone + Debug + KeyIdentifierImpl;
}


/// Structure representing key identifier.
/// 
/// Hides backend-specific details from user.
#[derive(Clone)]
pub struct KeyId<NativeId> {
    /// Engine-specific identifier
    id: NativeId
}


impl<NativeId> KeyId<NativeId> 
where
    NativeId: KeyIdentifier
{
    /// Creates a new key identifier from string reference.
    /// 
    /// * `id` - identifier as string
    pub fn new(id: &str) -> Self {
        KeyId { id: NativeId::create(id) }
    }
    
    /// Returns a backend-specific key identifier representation.
    pub(crate) fn native_id(&self) -> &NativeId {
        &self.id
    }
}


impl<NativeId> Display for KeyId<NativeId>
where
    NativeId: KeyIdentifier
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.native_id())
    }
}


/// Structure, that wraps a key handle.
pub struct Key<NativeHandle, NativeId> {
    /// Internal backend-specific key handle
    key: NativeHandle,

    /// Copy of key identifier
    id: KeyId<NativeId>
}


impl<NativeHandle, NativeId> Key<NativeHandle, NativeId> 
where
    NativeHandle: KeyHandle,
    NativeId: KeyIdentifier
{
    /// Creates a key handle from native key handle and identifier.
    /// 
    /// * `key` - native key handle
    /// * `id` - key identifier
    pub(crate) fn new(key: NativeHandle, id: &KeyId<NativeId>) -> Self {
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
    /// Key MUST NOT be expired, revoked nor disabled, and MUST be able 
    /// to perform encryption.
    pub(crate) fn is_suitable(&self) -> bool {
        let is_good = self.key.is_good();
        let can_encrypt = self.key.can_encrypt();

        is_good && can_encrypt
    }

    /// Returns key identifier
    pub fn id(&self) -> &KeyId<NativeId> {
        &self.id
    }
}
