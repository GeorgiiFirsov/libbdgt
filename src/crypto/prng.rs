use rand::{SeedableRng, RngCore};

use crate::error::{Result, Error};


/// Type of RNG used in bdgt.
type Rng = rand::rngs::StdRng;


/// Create an instance of RNG using system entropy.
pub(crate) fn create_prng() -> Rng {
    Rng::from_entropy()
}


/// Fill a buffer with random bytes.
/// 
/// * `buffer` - buffer to write random bytes
pub(crate) fn generate_random(buffer: &mut [u8]) -> Result<()> {
    create_prng()
        .try_fill_bytes(buffer)
        .map_err(Error::from)
}
