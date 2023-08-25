use std::fmt::{Display, Formatter, Debug};


/// Key identifier trait.
/// 
/// Implemented for [`KeyId`] and concrete engine-specific identifiers.
/// Defines functions, that maps identifier onto a string and vice versa. 
pub trait KeyIdentifier {
    /// Creates an identifier from string reference.
    /// 
    /// * `id` - identifier as string
    fn from_str(id: &str) -> Self;

    /// Converts identifier into a string.
    fn as_string(&self) -> String;
}


/// Key handle trait.
/// 
/// Implemented for concrete engine-specific handles.
pub trait KeyHandle {
    /// Checks if key is not expired, not revoked and not disabled.
    fn is_good(&self) -> bool;

    /// Checks if key is suitable for encryption.
    fn can_encrypt(&self) -> bool;
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
    NativeId: Clone + Debug + KeyIdentifier
{
    /// Returns a backend-specific key identifier representation.
    pub(crate) fn native_id(&self) -> &NativeId {
        &self.id
    }
}


impl<NativeId> Display for KeyId<NativeId>
where
    NativeId: Clone + Debug + KeyIdentifier
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.native_id())
    }
}


impl<NativeId> KeyIdentifier for KeyId<NativeId> 
where
    NativeId: Clone + Debug + KeyIdentifier
{
    fn from_str(id: &str) -> Self {
        KeyId { id: NativeId::from_str(id) }
    }

    fn as_string(&self) -> String {
        self.native_id()
            .as_string()
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
    NativeId: Clone + Debug + KeyIdentifier
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
