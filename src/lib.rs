/// # FFI
/// The `encode` function is exported as C ABI for integration with Python via ctypes or cffi.
/// It takes a pointer to a byte array and its length, and returns the average byte value.
#[unsafe(no_mangle)]
pub extern "C" fn encode(data: *const u8, len: usize) -> u8 {
    if data.is_null() || len <= 0 {
        return 0;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let sum: usize = slice.iter().map(|&b| b as usize).sum();
    return (sum / len) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let bytes = [10u8, 20, 30, 40];
        let avg = encode(bytes.as_ptr(), bytes.len());
        assert_eq!(avg, 25);
    }
}
