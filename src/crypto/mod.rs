mod engine;
mod key;
mod gpg_engine;
mod buffer;

pub use self::engine::CryptoEngine;
pub use self::gpg_engine::GpgCryptoEngine;
pub use self::key::{Key, KeyId};


/// Error message for missing secret key.
const MISSING_SECRET_KEY: &str = "Secret key is missing";

/// Error message for invalid key.
const KEY_IS_NOT_SUITABLE: &str = "Key is not suitable for bdgt";

/// Error message for encryption error.
const ENCRYPTION_ERROR: &str = "An error occurred during encryption";

/// Error message for decryption error.
const DECRYPTION_ERROR: &str = "An error occurred during decryption";
