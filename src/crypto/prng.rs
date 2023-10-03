use rand::{SeedableRng, RngCore, CryptoRng};

use crate::error::{Result, Error};


/// Type of RNG used in bdgt. It is a secure RNG, and
/// hence implements [`CryptoRng`].
pub(crate) struct Prng(rand::rngs::StdRng);


impl Prng {
    /// Create an instance of RNG using system entropy.
    pub fn new() -> Prng {
        Prng(rand::rngs::StdRng::from_entropy())
    }

    /// Fill a buffer with random bytes.
    /// 
    /// * `buffer` - buffer to write random bytes
    pub fn generate(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.0
            .try_fill_bytes(buffer)
            .map_err(Error::from)
    }
}


impl CryptoRng for Prng {}


impl RngCore for Prng {
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> std::result::Result<(), rand::Error> {
        self.0.try_fill_bytes(dest)
    }
}
