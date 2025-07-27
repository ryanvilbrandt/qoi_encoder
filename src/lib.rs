#[repr(u8)]
#[derive(PartialEq)]
pub enum EncodeError {
    None = 0,
    NullData = 1,
    InvalidDimensions = 2,
    InvalidChannels = 3,
}

#[repr(C)]
pub struct EncodeResult {
    pub ptr: *mut u8,
    pub len: usize,
    pub error: u8,
}

/// # FFI
/// The `encode` function is exported as C ABI for integration with Python via ctypes or cffi.
/// It takes a pointer to a byte array and its length, and returns a struct containing a pointer to
/// a bytestring, its length, and an error code.
#[unsafe(no_mangle)]
pub extern "C" fn encode(
    data: *const u8,
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
) -> EncodeResult {
    let error = check_for_invalid_input(data, width, height, channels, colorspace);
    if error != EncodeError::None {
        return return_error(error as u8);
    }

    // Convert the input data to a slice
    // let len = width * height * channels;
    // let slice = unsafe { std::slice::from_raw_parts(data, len) };

    // Create the output header and start the output vector
    let mut out = get_header(width, height, channels, colorspace);

    // Encode the data into the output vector
    let out_len = out.len();
    let boxed = out.into_boxed_slice();
    let ptr = Box::into_raw(boxed) as *mut u8;
    EncodeResult {
        ptr,
        len: out_len,
        error: EncodeError::None as u8,
    }
}

fn check_for_invalid_input(
    data: *const u8,
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
) -> EncodeError {
    if data.is_null() {
        return EncodeError::NullData;
    }
    if width == 0 || height == 0 {
        return EncodeError::InvalidDimensions;
    }
    if channels < 3 || channels > 4 {
        return EncodeError::InvalidChannels;
    }
    if colorspace > 1 {
        return EncodeError::InvalidChannels;
    }
    EncodeError::None
}

fn get_header(width: u32, height: u32, channels: u8, colorspace: u8) -> Vec<u8> {
    let mut header = Vec::with_capacity(14);
    header.extend_from_slice(b"qoif");
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.push(channels);
    header.push(colorspace);
    header
}

fn return_error(error: u8) -> EncodeResult {
    EncodeResult {
        ptr: std::ptr::null_mut(),
        len: 0,
        error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_encoded(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let bytes = [
            0x00, 0x00, 0x00,   0xFF, 0x00, 0x00,
            0xFF, 0xFF, 0x00,   0xFF, 0xFF, 0xFF,
        ];
        let result = encode(bytes.as_ptr(), 2, 2, 3, 0);
        assert!(!result.ptr.is_null());
        assert!(result.len > 0);
        assert_eq!(result.error, 0);
        free_encoded(result.ptr, result.len);
    }
}
