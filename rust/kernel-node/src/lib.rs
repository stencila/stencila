use kernel::formats::Format;
use kernel_micro::{include_file, MicroKernel};

/// A microkernel for Node
pub fn new() -> MicroKernel {
    MicroKernel::new(
        "node-micro",
        &[Format::JavaScript],
        true,
        false,
        false,
        ("node", "*"),
        &["{{script}}"],
        include_file!("node-kernel.js"),
        &[include_file!("node-codec.js")],
        "{{name}} = decodeValue({{json}})",
        "{{name}}",
        None,
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
    #[tokio::test]
    async fn basics() -> Result<()> {
        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        // Assign a variable and output it
        let (outputs, messages) = kernel
            .exec("const a = 2\na", Format::JavaScript, None)
            .await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2]);

        // Print the variable twice and then output it
        let (outputs, messages) = kernel
            .exec(
                "console.log(a)\nconsole.log(a)\na",
                Format::JavaScript,
                None,
            )
            .await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2, 2, 2]);

        // Syntax error
        let (outputs, messages) = kernel
            .exec("bad ^ # syntax", Format::JavaScript, None)
            .await?;
        assert_json_eq!(messages[0].error_type, "SyntaxError");
        assert_json_eq!(messages[0].error_message, "Invalid or unexpected token");
        assert!(messages[0].stack_trace.is_some());
        assert_json_eq!(outputs, json!([]));

        // Runtime error
        let (outputs, messages) = kernel.exec("foo", Format::JavaScript, None).await?;
        assert_json_eq!(messages[0].error_type, "ReferenceError");
        assert_json_eq!(messages[0].error_message, "foo is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_json_eq!(outputs, json!([]));

        // Set and get another variable
        kernel.set("b", Node::Integer(3)).await?;
        let b = kernel.get("b").await?;
        assert_json_eq!(b, 3);

        // Use both variables
        let (outputs, messages) = kernel.exec("a*b", Format::JavaScript, None).await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [6]);

        Ok(())
    }

    // Test that `console.log` arguments are treated as separate outputs
    #[tokio::test]
    async fn console_log() -> Result<()> {
        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        let (outputs, messages) = kernel
            .exec("console.log(1)", Format::JavaScript, None)
            .await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [1]);

        let (outputs, messages) = kernel
            .exec("console.log(1, 2, 3, 4)", Format::JavaScript, None)
            .await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [1, 2, 3, 4]);

        let (outputs, messages) = kernel
            .exec("console.log([1, 2, 3], 4, 'str')", Format::JavaScript, None)
            .await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, json!([[1, 2, 3], 4, "str"]));

        Ok(())
    }

    // Test that `console.debug`, `console.warn` etc are treated as separate messages
    #[tokio::test]
    async fn console_messages() -> Result<()> {
        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        let (outputs, messages) = kernel
            .exec(
                r#"
console.log(1)
console.debug("Debug message")
console.log(2)
console.info("Info message")
console.log(3)
console.warn("Warn message")
console.log(4)
console.error("Error message")
5
"#,
                Format::JavaScript,
                None,
            )
            .await?;

        assert_json_eq!(
            messages,
            json!([{
                "type": "CodeError",
                "errorType": "Debug",
                "errorMessage": "Debug message",
            }, {
                "type": "CodeError",
                "errorType": "Info",
                "errorMessage": "Info message",
            }, {
                "type": "CodeError",
                "errorType": "Warning",
                "errorMessage": "Warn message",
            } , {
                "type": "CodeError",
                "errorType": "Error",
                "errorMessage": "Error message",
            }])
        );
        assert_json_eq!(outputs, json!([1, 2, 3, 4, 5]));

        Ok(())
    }

    /// Test setting and getting of vars of different types
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

    /// Test re-declarations of variables
    #[tokio::test]
    async fn redeclarations() -> Result<()> {
        let mut kernel = new();
        match kernel.is_available().await {
            true => kernel.start_here().await?,
            false => return Ok(()),
        }

        // A variable declared with `var`

        let (outputs, messages) = kernel
            .exec("var a = 1\na", Format::JavaScript, None)
            .await?;
        assert_eq!(messages, vec![]);
        assert_json_eq!(outputs[0], json!(1));

        let (outputs, messages) = kernel
            .exec("var a = 2\na", Format::JavaScript, None)
            .await?;
        assert_eq!(messages, vec![]);
        assert_json_eq!(outputs[0], json!(2));

        let (outputs, messages) = kernel
            .exec("let a = 3\na", Format::JavaScript, None)
            .await?;
        assert_eq!(messages, vec![]);
        assert_json_eq!(outputs[0], json!(3));

        let (outputs, messages) = kernel
            .exec("const a = 4\na", Format::JavaScript, None)
            .await?;
        assert_eq!(messages, vec![]);
        assert_json_eq!(outputs[0], json!(4));

        // A variable declared with `let`

        let (outputs, messages) = kernel
            .exec("let b = 1\nb", Format::JavaScript, None)
            .await?;
        assert_eq!(messages, vec![]);
        assert_json_eq!(outputs[0], json!(1));

        let (outputs, messages) = kernel
            .exec("let b = 2\nb", Format::JavaScript, None)
            .await?;
        assert_eq!(messages, vec![]);
        assert_json_eq!(outputs[0], json!(2));

        let (outputs, messages) = kernel.exec("b = 3\nb", Format::JavaScript, None).await?;
        assert_eq!(messages, vec![]);
        assert_json_eq!(outputs[0], json!(3));

        // A variable declared with `const`

        let (outputs, messages) = kernel
            .exec("const c = 1\nc", Format::JavaScript, None)
            .await?;
        assert_eq!(messages, vec![]);
        assert_json_eq!(outputs[0], json!(1));

        let (.., messages) = kernel
            .exec("const c = 2\nc", Format::JavaScript, None)
            .await?;
        assert_eq!(
            messages[0].error_message,
            "Assignment to constant variable."
        );

        let (.., messages) = kernel.exec("c = 3\nc", Format::JavaScript, None).await?;
        assert_eq!(
            messages[0].error_message,
            "Assignment to constant variable."
        );

        Ok(())
    }
}
