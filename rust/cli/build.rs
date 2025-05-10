fn main() {
    // Required to use Kuzu extensions
    // See https://docs.rs/kuzu/0.10.0/kuzu/index.html#using-extensions
    println!("cargo:rustc-link-arg=-rdynamic");
}
