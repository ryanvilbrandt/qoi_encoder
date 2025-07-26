//! Integration test: calls the Rust library from Python using ctypes.
//! This test assumes that a Python virtual environment is set up with the Pillow library installed.

use std::process::Command;
use std::env;
use std::path::PathBuf;

#[test]
fn test_python_add() {
    // Build the cdylib
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .status()
        .expect("Failed to build library");
    assert!(status.success());

    // Find the shared library
    let target_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("target")
        .join("release");
    #[cfg(target_os = "windows")]
    let libname = "qoi_encoder.dll";
    #[cfg(target_os = "linux")]
    let libname = "libqoi_encoder.so";
    #[cfg(target_os = "macos")]
    let libname = "libqoi_encoder.dylib";
    let libpath = target_dir.join(libname);

    assert!(libpath.exists(), "Shared library not found: {:?}", libpath);

    // Confirm that a Python virtual environment is set up at "./venv"
    let venv_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("venv");
    assert!(venv_path.exists(), "Python virtual environment not found at {:?}", venv_path);

    #[cfg(target_os = "windows")]
    let python_bin = r"tests\venv\Scripts\python.exe";
    #[cfg(target_os = "linux")]
    let python_bin = "tests/venv/bin/python";
    #[cfg(target_os = "macos")]
    let python_bin = "tests/venv/bin/python";

    // Confirm that the virtual environment contains the Pillow library
    let output = Command::new(python_bin)
        .arg("-m")
        .arg("pip")
        .arg("show")
        .arg("Pillow")
        .output()
        .expect("Failed to check for Pillow library in virtual environment");

    assert!(
        output.status.success(),
        "Pillow library not found in virtual environment: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Run the Python script
    let py_script_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("python_integration_test.py");

    let output = Command::new(python_bin)
        .arg(&py_script_path)
        .output()
        .expect("Failed to run Python script");

    assert!(
        output.status.success(),
        "Python script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    
    println!(
        "Python script output:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
