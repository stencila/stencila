//! Test functions designed to be reused by Microkernel implementation

use kernel::{common::eyre::Result, stencila_schema::Node, KernelTrait};

use crate::MicroKernel;

/// Test that string variables, including with combinations of quote characters,
/// and other special characters can be setted and getted.
///
/// Note that three single quotes within the string WILL still error with Python
/// microkernel so is excluded here.
pub async fn set_get_strings(kernel: &mut MicroKernel) -> Result<()> {
    for value in [
        "",
        "hello",
        "some spaces",
        "'",
        "''",
        "\"",
        "\"\"",
        "\"\"\"",
    ] {
        let value = Node::String(value.to_string());
        kernel.set("a", value.clone()).await?;
        let var = kernel.get("a").await?;
        assert_eq!(var, value);
    }
    Ok(())
}
