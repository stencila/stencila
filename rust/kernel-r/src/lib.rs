use kernel_micro::{include_file, MicroKernel};

pub fn new() -> MicroKernel {
    MicroKernel::new(
        "ur",
        &["r"],
        ("Rscript", "*"),
        &["{{script}}"],
        include_file!("r-kernel.r"),
        &[include_file!("r-codec.r")],
        "{{name}} <- decode_value(\"{{json}}\")",
        "{{name}}",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{eyre::Result, stencila_schema::Node, KernelTrait};
    use test_utils::{assert_json_eq, serde_json::json};

    /// Tests of basic functionality
    /// This test is replicated in all the microkernels.
    /// Other test should be written for language specific quirks and regressions.
    #[tokio::test]
    async fn basics() -> Result<()> {
        let mut kernel = new();
        if !kernel.available().await {
            return Ok(());
        } else {
            kernel.start().await?;
        }

        // Assign a variable and output it
        let (outputs, messages) = kernel.exec("a = 2\na").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [[2]]);

        // Print the variable twice and then output it
        let (outputs, messages) = kernel.exec("print(a)\nprint(a)\na").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [[2], [2], [2]]);

        // Syntax error
        let (outputs, messages) = kernel.exec("bad ^ # syntax").await?;
        println!("{:?}", messages);
        assert_json_eq!(messages[0].error_type, "SyntaxError");
        assert_json_eq!(
            messages[0].error_message,
            "<text>:2:0: unexpected end of input\n1: bad ^ # syntax\n   ^"
        );
        assert_json_eq!(outputs, json!([]));

        // Runtime error
        let (outputs, messages) = kernel.exec("foo").await?;
        assert_json_eq!(messages[0].error_type, "RuntimeError");
        assert_json_eq!(messages[0].error_message, "object 'foo' not found");
        assert_json_eq!(outputs, json!([]));

        // Set and get another variable
        kernel.set("b", Node::Integer(3)).await?;
        let b = kernel.get("b").await?;
        assert_json_eq!(b, [3]);

        // Use both variables
        let (outputs, messages) = kernel.exec("a*b").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [[6]]);

        Ok(())
    }

    /// Test that an assignment on the last line does not generate an output
    #[tokio::test]
    async fn assignment_no_output() -> Result<()> {
        let mut kernel = new();
        if !kernel.available().await {
            return Ok(());
        } else {
            kernel.start().await?;
        }

        let (outputs, messages) = kernel.exec("a <- 1").await?;
        assert!(messages.is_empty());
        assert!(outputs.is_empty());

        let (outputs, messages) = kernel.exec("b = 2").await?;
        assert!(messages.is_empty());
        assert!(outputs.is_empty());

        let (outputs, messages) = kernel.exec("print(a)\nprint(b)\na_b <- a + b").await?;
        assert!(messages.is_empty());
        assert_json_eq!(outputs, [[1], [2]]);

        Ok(())
    }
}
