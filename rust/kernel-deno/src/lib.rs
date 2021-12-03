use kernel::eyre::Result;
use kernel_micro::{include_file, MicroKernel};

pub async fn new() -> Result<MicroKernel> {
    MicroKernel::new(
        "typescript",
        (
            "deno",
            ">1.7",
            &["run", "--quiet", "--unstable", "{{script}}"],
        ),
        include_file!("deno-kernel.ts"),
        &[include_file!("deno-codec.ts")],
        "{{name}} = JSON.parse('{{json}}')",
        "{{name}}",
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{stencila_schema::Node, KernelTrait};
    use test_utils::{assert_json_eq, serde_json::json};

    /// Tests of basic functionality
    /// This test is replicated in all the microkernels.
    /// Other test should be written for language specific quirks and regressions.
    #[tokio::test]
    async fn basics() -> Result<()> {
        let mut kernel = new().await?;
        kernel.start().await?;

        // Assign a variable and output it
        let (outputs, messages) = kernel.exec("const a = 2\na").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2]);

        // Print the variable twice and then output it
        let (outputs, messages) = kernel.exec("console.log(a)\nconsole.log(a)\na").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2, 2, 2]);

        // Syntax error
        let (outputs, messages) = kernel.exec("bad ^ # syntax").await?;
        //assert_json_eq!(messages[0].error_type, "SyntaxError");
        //assert_json_eq!(messages[0].error_message, "Invalid or unexpected token");
        assert!(messages[0].stack_trace.is_some());
        assert_json_eq!(outputs, json!([]));

        // Runtime error
        let (outputs, messages) = kernel.exec("foo").await?;
        assert_json_eq!(messages[0].error_type, "ReferenceError");
        assert_json_eq!(messages[0].error_message, "foo is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_json_eq!(outputs, json!([]));

        // Set and get another variable
        kernel.set("b", Node::Integer(3)).await?;
        let b = kernel.get("b").await?;
        assert_json_eq!(b, 3);

        // Use both variables
        let (outputs, messages) = kernel.exec("a*b").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [6]);

        Ok(())
    }

    // Test that `console.log` arguments are treated as separate outputs
    #[tokio::test]
    async fn console_log() -> Result<()> {
        let mut kernel = new().await?;
        kernel.start().await?;

        let (outputs, messages) = kernel.exec("console.log(1)").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [1]);

        let (outputs, messages) = kernel.exec("console.log(1, 2, 3, 4)").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [1, 2, 3, 4]);

        let (outputs, messages) = kernel.exec("console.log([1, 2, 3], 4, 'str')").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, json!([[1, 2, 3], 4, "str"]));

        Ok(())
    }
}
