//! Emit the shared fixture corpus for the linked wasm/native parity harness.
#[path = "../tests/support/mod.rs"]
mod support;

#[allow(clippy::print_stdout)]
fn main() -> eyre::Result<()> {
    for source in support::fixtures()? {
        println!("{}", serde_json::to_string(&source)?);
    }
    Ok(())
}
