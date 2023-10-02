mod key;
mod prng;
mod buffer;
mod engine;
mod symmetric;
mod gpg_engine;

pub use self::engine::CryptoEngine;
pub use self::buffer::CryptoBuffer;
pub use self::gpg_engine::GpgCryptoEngine;
pub use self::key::{Key, KeyId};

pub(crate) use self::key::KeyIdentifier;


/// Error message for missing secret key.
const MISSING_SECRET_KEY: &str = "Secret key is missing";

/// Error message for invalid key.
const KEY_IS_NOT_SUITABLE: &str = "Key is not suitable for bdgt";

/// Error message for invalid engine state.
const INVALID_ENGINE_STATE: &str = "Engine is in invalid state";

/// Error message for encryption error.
const ENCRYPTION_ERROR: &str = "An error occurred during encryption";

/// Error message for decryption error.
const DECRYPTION_ERROR: &str = "An error occurred during decryption";
