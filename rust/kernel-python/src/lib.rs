use kernel_micro::{include_file, MicroKernel};

/// A microkernel for Python
pub fn new() -> MicroKernel {
    MicroKernel::new(
        "python-micro",
        &["python"],
        &["linux", "macos", "windows"],
        &["linux", "macos"],
        ("python3", "*"),
        &["{{script}}"],
        include_file!("python_kernel.py"),
        &[include_file!("python_codec.py")],
        "{{name}} = decode_value(\"{{json}}\")",
        "{{name}}",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{
        eyre::{bail, Result},
        stencila_schema::Node,
        KernelTrait,
    };
    use test_utils::{assert_json_eq, serde_json::json, skip_ci_os};

    async fn skip_or_kernel() -> Result<MicroKernel> {
        if skip_ci_os("windows", "Failing on Windows CIs") {
            bail!("Skipping")
        }

        let mut kernel = new();
        if !kernel.available().await {
            eprintln!("Python not available on this machine");
            bail!("Skipping")
        } else {
            kernel.start().await?;
        }

        Ok(kernel)
    }

    /// Tests of basic functionality
    /// This test is replicated in all the microkernels.
    /// Other test should be written for language specific quirks and regressions.
    #[tokio::test]
    async fn basics() -> Result<()> {
        let mut kernel = match skip_or_kernel().await {
            Ok(kernel) => kernel,
            Err(..) => return Ok(()),
        };

        // Assign a variable and output it
        let (outputs, messages) = kernel.exec("a = 2\na").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2]);

        // Print the variable twice and then output it
        let (outputs, messages) = kernel.exec("print(a)\nprint(a)\na").await?;
        assert_json_eq!(messages, json!([]));
        assert_json_eq!(outputs, [2, 2, 2]);

        // Syntax error
        let (outputs, messages) = kernel.exec("bad ^ # syntax").await?;
        assert_json_eq!(messages[0].error_type, "SyntaxError");
        assert_json_eq!(messages[0].error_message, "invalid syntax (<code>, line 1)");
        assert!(messages[0].stack_trace.is_some());
        assert_json_eq!(outputs, json!([]));

        // Runtime error
        let (outputs, messages) = kernel.exec("foo").await?;
        assert_json_eq!(messages[0].error_type, "NameError");
        assert_json_eq!(messages[0].error_message, "name 'foo' is not defined");
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

    /// Test forking
    #[tokio::test]
    async fn fork() -> Result<()> {
        let mut kernel = match skip_or_kernel().await {
            Ok(kernel) => {
                if kernel.forkable().await {
                    kernel
                } else {
                    eprintln!("Not forkable on this OS");
                    return Ok(());
                }
            }
            Err(..) => return Ok(()),
        };

        // In the kernel import a module and assign a variable
        let (outputs, messages) = kernel
            .exec("from random import uniform as runif\nvar = runif(0, 1)\nvar")
            .await?;
        assert_json_eq!(messages, json!([]));
        assert_eq!(outputs.len(), 1);
        let var = outputs[0].clone();

        // Now fork-exec. The fork should be able to use the module and access the
        // variable but any change to variable should not change its value in the parent kernel
        let (outputs, messages) = kernel.fork_exec("print(var)\nvar = runif(0, 1)").await?;
        assert_json_eq!(messages, json!([]));
        assert_eq!(outputs.len(), 1);
        assert_json_eq!(outputs[0], var);

        // Back in the parent kernel, var should still have its original value
        assert_json_eq!(var, kernel.get("var").await?);
        let (outputs, messages) = kernel.exec("var").await?;
        assert_json_eq!(messages, json!([]));
        assert_eq!(outputs.len(), 1);

        Ok(())
    }
}
