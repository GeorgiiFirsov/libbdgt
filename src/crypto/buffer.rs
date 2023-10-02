/// Provides method of constructing a value with invalidating of source.
pub trait DestructiveFrom<T> {
    /// Construct object and invalidate source.
    fn destructive_from(value: &mut T) -> Self;
}

/// Struct for wrapping a sensitive data.
/// 
/// Implements [`core::ops::Drop`] trait, that erases internal 
/// data at destruction time.
pub struct CryptoBuffer {
    /// Raw internal data
    data: Vec<u8>
}


impl CryptoBuffer {
    /// Creates an empty buffer.
    pub fn new() -> Self {
        CryptoBuffer { data: Vec::new() }
    }

    /// Creates a buffer with specified amount of zeros.
    /// 
    /// * `size` - initial size of buffer
    pub fn new_with_size(size: usize) -> Self {
        CryptoBuffer { data: vec![0; size] }
    }

    /// Returns read-only raw bytes of the stored data.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns mutable raw bytes of the stored data.
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    /// Check if buffer is empty. 
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}


impl CryptoBuffer {
    fn destroy_data(data: &mut [u8]) {
        //
        // Just zero passed memory block
        //
    
        for e in data.iter_mut() {
            *e = 0u8;
        }
    }
}


impl Drop for CryptoBuffer {
    fn drop(&mut self) {
        Self::destroy_data(&mut self.data);
    }
}


impl Default for CryptoBuffer {
    fn default() -> Self {
        Self::new()
    }
}


impl From<Vec<u8>> for CryptoBuffer {
    fn from(value: Vec<u8>) -> Self {
        Self { data: value }
    }
}


impl From<&[u8]> for CryptoBuffer {
    fn from(value: &[u8]) -> Self {
        Self { data: Vec::from(value) }
    }
}


impl DestructiveFrom<String> for CryptoBuffer {
    fn destructive_from(value: &mut String) -> Self {
        let buffer = Self{ data: Vec::from(value.as_bytes()) };
        
        //
        // Destroy source and return constructed buffer
        //

        unsafe { Self::destroy_data(value.as_bytes_mut()) };
        buffer
    }
}
