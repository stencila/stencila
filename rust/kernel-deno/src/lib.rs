use kernel_micro::{include_file, MicroKernel};

/// A microkernel for Deno
pub fn new() -> MicroKernel {
    MicroKernel::new(
        "deno-micro",
        &["javascript", "typescript"],
        true,
        false,
        false,
        ("deno", ">=1.7"),
        &["run", "--quiet", "--unstable", "{{script}}"],
        include_file!("deno-kernel.ts"),
        &[include_file!("deno-codec.ts")],
        "{{name}} = decodeValue({{json}})",
        "{{name}}",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{
        common::{eyre::Result, tokio},
        stencila_schema::Node,
        KernelTrait,
    };
    use test_utils::{assert_json_eq, common::serde_json::json};

    /// Tests of basic functionality
    /// This test is replicated in all the microkernels.
    /// Other test should be written for language specific quirks and regressions.
    #[ignore]
    #[tokio::test]
    async fn basics() -> Result<()> {
        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        // Assign a variable and output it
        let (outputs, messages) = kernel.exec("const a = 2\na", None).await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2]);

        // Print the variable twice and then output it
        let (outputs, messages) = kernel
            .exec("console.log(a)\nconsole.log(a)\na", None)
            .await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2, 2, 2]);

        // Syntax error
        let (outputs, messages) = kernel.exec("bad ^ # syntax", None).await?;
        //assert_json_eq!(messages[0].error_type, "SyntaxError");
        //assert_json_eq!(messages[0].error_message, "Invalid or unexpected token");
        assert!(messages[0].stack_trace.is_some());
        assert_json_eq!(outputs, json!([]));

        // Runtime error
        let (outputs, messages) = kernel.exec("foo", None).await?;
        assert_json_eq!(messages[0].error_type, "ReferenceError");
        assert_json_eq!(messages[0].error_message, "foo is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_json_eq!(outputs, json!([]));

        // Set and get another variable
        kernel.set("b", Node::Integer(3)).await?;
        let b = kernel.get("b").await?;
        assert_json_eq!(b, 3);

        // Use both variables
        let (outputs, messages) = kernel.exec("a*b", None).await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [6]);

        Ok(())
    }

    // Test that `console.log` arguments are treated as separate outputs
    #[ignore]
    #[tokio::test]
    async fn console_log() -> Result<()> {
        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        let (outputs, messages) = kernel.exec("console.log(1)", None).await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [1]);

        let (outputs, messages) = kernel.exec("console.log(1, 2, 3, 4)", None).await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [1, 2, 3, 4]);

        let (outputs, messages) = kernel
            .exec("console.log([1, 2, 3], 4, 'str')", None)
            .await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, json!([[1, 2, 3], 4, "str"]));

        Ok(())
    }

    /// Test setting and getting of vars of different types
    #[ignore]
    #[tokio::test]
    async fn set_get_vars() -> Result<()> {
        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        kernel_micro::tests::set_get_strings(&mut kernel).await?;

        Ok(())
    }
}
