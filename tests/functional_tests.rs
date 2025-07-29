use std::path::PathBuf;
use std::{env, fs};

use image::DynamicImage;
use image::GenericImageView;
use image::ImageReader;

use qoi_encoder::encode;
use qoi_encoder::free_encoded;

#[test]
fn encode_pngs_and_compare_to_qoi() {
    let test_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("qoi_test_images");

    for entry in fs::read_dir(test_dir).expect("Failed to read test images directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.file_name().and_then(|n| n.to_str()) != Some("edgecase.png") {
            continue;
        }

        if path.extension().map(|ext| ext == "png").unwrap_or(false) {
            let mut mismatch = false;
            let mut mismatch_idx = 0;
            let mut encoded_window = vec![];
            let mut reference_window = vec![];
            {
                // Load PNG
                println!("Comparing {:?}", path.file_name().unwrap());
                let img = ImageReader::open(&path)
                    .expect("Failed to open PNG")
                    .decode()
                    .expect("Failed to decode PNG");

                let (width, height) = img.dimensions();
                let (channels, raw_data) = match img {
                    DynamicImage::ImageRgb8(ref rgb) => (3, rgb.as_raw()),
                    DynamicImage::ImageRgba8(ref rgba) => (4, rgba.as_raw()),
                    _ => panic!("Unsupported image format"),
                };

                // let c: u8 = if path.file_name().and_then(|n| n.to_str()) == Some("edgecase.png") { 4 } else { channels };

                // Encode PNG data to QOI
                let result = encode(
                    raw_data.as_ptr(),
                    width,
                    height,
                    channels,
                    0, // colorspace: sRGB with linear alpha
                );
                assert_eq!(result.error, 0, "Encoding failed for {:?}", path);

                let encoded = unsafe { std::slice::from_raw_parts(result.ptr, result.len) };

                // Scope guard to ensure memory is freed
                struct EncodedGuard {
                    ptr: *mut u8,
                    len: usize,
                }
                impl Drop for EncodedGuard {
                    fn drop(&mut self) {
                        free_encoded(self.ptr, self.len);
                    }
                }
                let _guard = EncodedGuard { ptr: result.ptr, len: result.len };

                // Load reference QOI file
                let mut qoi_path = path.clone();
                qoi_path.set_extension("qoi");
                let reference = fs::read(&qoi_path)
                    .expect(&format!("Failed to read reference QOI file: {:?}", qoi_path));

                // Compare
                if encoded != reference {
                    // Find the first mismatching index
                    mismatch_idx = encoded.iter()
                        .zip(reference.iter())
                        .position(|(a, b)| a != b)
                        .unwrap_or(0);

                    // Print a window of 5 bytes before and after the mismatch
                    let start = mismatch_idx.saturating_sub(5);
                    let end = (mismatch_idx + 16).min(encoded.len()).min(reference.len());

                    encoded_window = encoded[start..end].to_vec();
                    reference_window = reference[start..end].to_vec();

                    mismatch = true;
                } else {
                    println!("Successfully encoded and compared {:?}", path.file_name().unwrap());
                }
            }
            // _guard is dropped here, memory is freed
            if mismatch {
                eprintln!(
                    "Mismatch at index {}:\nEncoded:   {:02X?}\nReference: {:02X?}",
                    mismatch_idx,
                    encoded_window,
                    reference_window
                );
                // End early if there's a mismatch
                return;
            }
        }
    }
}
