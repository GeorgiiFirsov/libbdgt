mod engine;
mod key;

pub use self::engine::{CryptoEngine};
pub use self::key::{Key, KeyId, ExportedKey};


/// Error message for missing secret key.
const MISSING_SECRET_KEY: &str = "Secret key is missing";

/// Error message for invalid key.
const KEY_IS_NOT_SUITABLE: &str = "Key is not suitable for bdgt";