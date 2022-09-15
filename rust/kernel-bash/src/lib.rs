use kernel::formats::Format;
use kernel_micro::{include_file, MicroKernel};

/// A microkernel for Bash
///
/// The `kernel-bash.sh` script relies on `/dev/stdin`, `/dev/stdout`,
/// and `/dev/stderr` so this kernel is not available on Windows.
pub fn new() -> MicroKernel {
    MicroKernel::new(
        "bash-micro",
        &[Format::Bash, Format::Shell],
        true,
        false,
        false,
        ("bash", "*"),
        &["{{script}}"],
        include_file!("bash-kernel.sh"),
        &[],
        "{{name}}={{json}}",
        "echo ${{name}}",
        None
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{
        common::{
            eyre::{bail, Result},
            tokio,
        },
        stencila_schema::Node,
        KernelTrait,
    };
    use test_utils::{assert_json_eq, common::serde_json::json, skip_ci_os};

    /// Tests of basic functionality
    /// This test is replicated in all the microkernels.
    /// Other test should be written for language specific quirks and regressions.
    #[tokio::test]
    async fn basics() -> Result<()> {
        if skip_ci_os("windows", "test currently failing on Windows CI") {
            return Ok(());
        }
        if skip_ci_os("macos", "test currently failing on MacOS CI") {
            return Ok(());
        }

        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        // Assign a variable and output it
        let (outputs, messages) = kernel.exec("a=2\necho $a\n", Format::Bash, None).await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2]);

        // Syntax error
        let (outputs, messages) = kernel.exec("if", Format::Bash, None).await?;
        assert!(messages[0]
            .error_message
            .ends_with("syntax error: unexpected end of file"));
        assert_json_eq!(outputs, json!([]));

        // Runtime error
        let (outputs, messages) = kernel.exec("foo", Format::Bash, None).await?;
        assert!(messages[0]
            .error_message
            .ends_with("foo: command not found"));
        assert_json_eq!(outputs, json!([]));

        // Set and get another variable
        kernel.set("b", Node::Integer(3)).await?;
        let b = kernel.get("b").await?;
        assert_json_eq!(b, 3);

        // Use both variables
        let (outputs, messages) = kernel.exec("echo $a$b\n", Format::Bash, None).await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [23]);

        Ok(())
    }

    #[tokio::test]
    async fn percent_escaping() -> Result<()> {
        if skip_ci_os("windows", "test currently failing on Windows CI") {
            return Ok(());
        }

        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        let (outputs, messages) = kernel.exec("date +%s", Format::Bash, None).await?;
        assert_json_eq!(messages, json!([]));
        let timestamp = outputs.first().unwrap();
        match timestamp {
            Node::Integer(timestamp) => assert!(*timestamp > 1600000000),
            _ => bail!("Expected an integer"),
        }

        Ok(())
    }
}
