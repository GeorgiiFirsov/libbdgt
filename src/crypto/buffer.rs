/// Struct for wrapping a sensitive data.
/// 
/// Implements [`core::ops::Drop`] trait, that erases internal 
/// data at destruction time.
pub struct CryptoBuffer {
    /// Raw internal data
    data: Vec<u8>
}


impl CryptoBuffer {
    /// Creates a buffer from vector by moving it into a new object.
    /// 
    /// * `data` - raw data bytes
    pub(crate) fn new(data: Vec<u8>) -> Self {
        CryptoBuffer { data: data }
    }

    /// Returns read-only raw bytes of the stored data.
    pub fn as_raw(&self) -> &[u8] {
        &self.data
    }
}


impl Drop for CryptoBuffer {
    fn drop(&mut self) {
        //
        // Just zero stored memory
        //
        for e in self.data.iter_mut() {
            *e = 0u8;
        }
    }
}
