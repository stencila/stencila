fn main() {
    println!("cargo:rerun-if-changed=src/catalog/models.json");
}
