// This build script sets environment variables to force single-threaded test execution
// Tests in this crate modify the current working directory and access the user config
// directory, which can cause race conditions when run in parallel.

fn main() {
    // Force tests to run single-threaded by default
    println!("cargo:rustc-env=RUST_TEST_THREADS=1");
}
