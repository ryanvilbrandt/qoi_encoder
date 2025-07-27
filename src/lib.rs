#[repr(u8)]
#[derive(PartialEq)]
pub enum EncodeError {
    None = 0,
    NullData = 1,
    InvalidDimensions = 2,
    InvalidChannels = 3,
    InvalidColorspace = 4,
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
    let len = width * height * channels as u32;
    let slice = unsafe { std::slice::from_raw_parts(data, len as usize) };

    // Create the output header and start the output vector
    let out = encode_image_data(slice, width, height, channels, colorspace);

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
        return EncodeError::InvalidColorspace;
    }
    EncodeError::None
}

fn encode_image_data(
    data: &[u8],
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
) -> Vec<u8> {
    let mut out = get_header(width, height, channels, colorspace);
    let mut index_array: [Vec<u8>; 64] = std::array::from_fn(|_| Vec::new());
    let mut last_pixel: Vec<u8> = Vec::new();
    let mut run_length: u8 = 0;

    for chunk in data.chunks_exact(channels as usize) {
        let pixel = chunk.to_vec();
        let index: usize = get_pixel_index(&pixel);
        println!("{:?}", pixel);
        // Initial check for when last_pixel is empty
        if last_pixel.is_empty() {
            add_pixel(&mut out, &pixel);
            add_to_index(&mut index_array, index, &pixel);
            last_pixel = pixel;
            continue;
        }
        // Encode a run
        if last_pixel == pixel {
            run_length += 1;
            if run_length == 62 {
                add_run(&mut out, run_length);
                run_length = 0;
            }
            continue;
        }
        if run_length > 0 {
            // The current pixel has broken the run, so encode the run length now
            add_run(&mut out, run_length);
            run_length = 0;
            // Keep going because we still need to encode the current pixel
        }
        // Encode index
        if index_array[index] == pixel {
            add_index(&mut out, index);
            continue;
        }
        // Encode diff
        if !encode_diff(&mut out, &last_pixel, &pixel) {
            add_pixel(&mut out, &pixel);
        }
        add_to_index(&mut index_array, index, &pixel);
    }
    return out;
}

fn get_header(width: u32, height: u32, channels: u8, colorspace: u8) -> Vec<u8> {
    // Set the capacity to match the size of the original data. By setting the capacity to an
    // upper bound, we can avoid reallocations during the encoding process and improve performance
    // at the cost of some memory overhead. But because this memory is equivalent to the size of
    // the original data, it is not a significant overhead.
    let mut header = Vec::with_capacity((width * height * channels as u32) as usize);
    header.extend_from_slice(b"qoif");
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.push(channels);
    header.push(colorspace);
    header
}

fn get_pixel_index(pixel: &Vec<u8>) -> usize {
    let mut index = pixel[0] as usize * 3 + pixel[1] as usize * 5 + pixel[2] as usize * 7;
    if pixel.len() == 4 {
        index += pixel[3] as usize * 11;
    }
    return index % 64
}

fn add_to_index(index_array: &mut [Vec<u8>; 64], index: usize, pixel: &Vec<u8>) {
    index_array[index] = pixel.clone();
}

fn add_pixel(out: &mut Vec<u8>, pixel: &Vec<u8>) {
    if pixel.len() == 3 {
        out.push(0b1111_1110); // RGB
    } else {
        out.push(0b1111_1111); // RGBA
    }
    out.extend_from_slice(&pixel);
}

fn add_run(out: &mut Vec<u8>, run_length: u8) {
    out.push(0b1100_0000 | run_length);
}

fn add_index(out: &mut Vec<u8>, index: usize) {
    out.push(index as u8);
}

fn encode_diff(out: &mut Vec<u8>, pixel: &Vec<u8>, last_pixel: &Vec<u8>) -> bool {
    // Check that alpha is unchanged. If it changed, skip this function.
    if pixel.len() == 4 && pixel[3] != last_pixel[3] {
        return false;
    }

    let dr = pixel[0].wrapping_sub(last_pixel[0]);
    let dg = pixel[1].wrapping_sub(last_pixel[1]);
    let db = pixel[2].wrapping_sub(last_pixel[2]);

    // Small diff: -2..=1 as u8: 254..=1
    let small = |v: u8| v >= 254 || v <= 1;
    if small(dr) && small(dg) && small(db) {
        let byte = 0b0100_0000
            | (((dr.wrapping_add(2)) & 0x03) << 4)
            | (((dg.wrapping_add(2)) & 0x03) << 2)
            | ((db.wrapping_add(2)) & 0x03);
        out.push(byte);
        return true;
    }

    let dr_dg = dr.wrapping_sub(dg);
    let db_dg = db.wrapping_sub(dg);

    // Luma diff:
    // -32..=31 as u8: 224..=31
    //  -8..=7  as u8: 248..=7
    let luma = |v: u8| v >= 224 || v <= 31;
    let luma_small = |v: u8| v >= 248 || v <= 7;
    if luma(dg) && luma_small(dr_dg) && luma_small(db_dg) {
        out.push(0b1000_0000 | (dg.wrapping_add(32) & 0x3F));
        out.push(((dr_dg.wrapping_add(8) & 0x0F) << 4) | (db_dg.wrapping_add(8) & 0x0F));
        return true;
    }

    return false;
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
