use image::ImageReader;
use std::path::Path;

#[test]
fn test_print_pixel() {
    // Example arguments
    let image_path = "tests/qoi_test_images/dice.png";
    let pixel_index: usize = 24561;

    // Load the image
    let img = ImageReader::open(&Path::new(image_path))
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image")
        .to_rgba8();

    let raw = img.as_raw();

    // Each pixel is 4 bytes (RGBA)
    let start = pixel_index * 4;
    assert!(start + 4 <= raw.len(), "Pixel index out of bounds");

    let rgba = &raw[start..start + 4];
    println!(
        "Pixel {} RGBA: R={:02X}, G={:02X}, B={:02X}, A={:02X}",
        pixel_index, rgba[0], rgba[1], rgba[2], rgba[3]
    );
}
