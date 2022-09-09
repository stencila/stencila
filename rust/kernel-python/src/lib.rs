use kernel_micro::{include_file, MicroKernel};

/// A microkernel for Python
pub fn new() -> MicroKernel {
    MicroKernel::new(
        "python-micro",
        &["python"],
        true,
        cfg!(not(target_os = "windows")),
        cfg!(any(target_os = "linux", target_os = "macos")),
        ("python3", "*"),
        &["{{script}}"],
        include_file!("python_kernel.py"),
        &[include_file!("python_codec.py")],
        "{{name}} = __decode_value__(r'''{{json}}''')",
        "{{name}}",
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
        KernelTrait, TaskResult,
    };
    use test_utils::{assert_json_eq, assert_json_is, skip_ci_os};

    async fn skip_or_kernel() -> Result<MicroKernel> {
        if skip_ci_os("windows", "Failing on Windows CIs") {
            bail!("Skipping")
        }

        let mut kernel = new();
        if !kernel.is_available().await {
            eprintln!("Python not available on this machine");
            bail!("Skipping")
        } else {
            kernel.start_here().await?;
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

        // The execution context should start off empty
        let (outputs, messages) = kernel.exec("dir()", None).await?;
        assert_json_is!(messages, []);
        assert_json_is!(outputs[0], ["__builtins__", "__decode_value__", "print"]);

        // Assign a variable and output it
        let (outputs, messages) = kernel.exec("a = 2\na", None).await?;
        assert_json_is!(messages, []);
        assert_json_is!(outputs, [2]);

        // The execution context should now have the var
        let (outputs, messages) = kernel.exec("dir()", None).await?;
        assert_json_is!(messages, []);
        assert_json_is!(
            outputs[0],
            ["__builtins__", "__decode_value__", "a", "print"]
        );

        // Print the variable twice and then output it
        let (outputs, messages) = kernel.exec("print(a)\nprint(a)\na", None).await?;
        assert_json_is!(messages, []);
        assert_json_is!(outputs, [2, 2, 2]);

        // Syntax error
        let (outputs, messages) = kernel.exec("bad ^ # syntax", None).await?;
        assert_json_is!(messages[0].error_type, "SyntaxError");
        assert_json_is!(messages[0].error_message, "invalid syntax (<code>, line 1)");
        assert!(messages[0].stack_trace.is_some());
        assert_json_is!(outputs, []);

        // Runtime error
        let (outputs, messages) = kernel.exec("foo", None).await?;
        assert_json_is!(messages[0].error_type, "NameError");
        assert_json_is!(messages[0].error_message, "name 'foo' is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_json_is!(outputs, []);

        // Set and get another variable
        kernel.set("b", Node::Integer(3)).await?;
        let b = kernel.get("b").await?;
        assert_json_is!(b, 3);

        // Use both variables
        let (outputs, messages) = kernel.exec("a*b", None).await?;
        assert_json_is!(messages, []);
        assert_json_is!(outputs, [6]);

        Ok(())
    }

    /// Test interrupting a task
    #[tokio::test]
    async fn exec_async() -> Result<()> {
        let mut kernel = match skip_or_kernel().await {
            Ok(kernel) => {
                if kernel.is_interruptable().await {
                    kernel
                } else {
                    eprintln!("Not interruptable on this OS");
                    return Ok(());
                }
            }
            Err(..) => return Ok(()),
        };

        // Start a long running task in the kernel that should get interrupted
        let mut task = kernel
            .exec_async(
                "import time\nstarted = True\ntime.sleep(10)\nfinished = True",
                None,
            )
            .await?;

        // Sleep a little to allow the task to start, then interrupt it
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        task.interrupt().await?;

        // Check that was started but not finished
        let (outputs, messages) = kernel
            .exec("[started, 'finished' in locals()]", None)
            .await
            .unwrap();
        assert_json_is!(messages, []);
        assert_json_is!(outputs, [[true, false]]);

        Ok(())
    }

    /// Test forking
    #[tokio::test]
    async fn forking() -> Result<()> {
        let mut kernel = match skip_or_kernel().await {
            Ok(kernel) => {
                if kernel.is_forkable().await {
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
            .exec(
                "from random import uniform as runif\nvar = runif(0, 1)\nvar",
                None,
            )
            .await?;
        assert_json_is!(messages, []);
        assert_eq!(outputs.len(), 1);
        let var = outputs[0].clone();

        // Now fork-exec. The fork should be able to use the module and access the
        // variable but any change to variable should not change its value in the parent kernel
        let mut task = kernel
            .exec_fork("print(var)\nvar = runif(0, 1)", None)
            .await?;
        let TaskResult { outputs, messages } = task.result().await?;
        assert_json_is!(messages, []);
        assert_eq!(outputs.len(), 1);
        assert_json_is!(outputs[0], var);

        // Back in the parent kernel, var should still have its original value
        assert_json_eq!(var, kernel.get("var").await?);
        let (outputs, messages) = kernel.exec("var", None).await?;
        assert_json_is!(messages, []);
        assert_eq!(outputs.len(), 1);

        // Now create a persistent fork and ensure were can execute multiple tasks in it including
        // getting the original var and using the imported `runif` function
        let mut fork = kernel.create_fork("").await?;
        let (outputs, messages) = fork.exec("var", None).await?;
        assert_json_is!(messages, []);
        assert_json_is!(outputs, [var]);
        let (outputs, messages) = fork.exec("runif(0, 1)", None).await?;
        assert_json_is!(messages, []);
        assert_eq!(outputs.len(), 1);

        Ok(())
    }

    /// Test that imported modules are available in functions
    ///
    /// This is a regression test for a bug found during usage.
    /// Before the associated fix would get error "name 'time' is not defined"
    #[tokio::test]
    async fn imports() -> Result<()> {
        let mut kernel = match skip_or_kernel().await {
            Ok(kernel) => kernel,
            Err(..) => return Ok(()),
        };

        // Import a module and a function from another module in one task
        let (outputs, messages) = kernel
            .exec("import time\nfrom datetime import datetime", None)
            .await?;
        assert_json_is!(messages, []);
        assert_json_is!(outputs, []);

        // Check that both can be used from within a function in another task
        let (outputs, messages) = kernel
            .exec(
                "def func():\n\treturn (time.time(), datetime.now())\nfunc()",
                None,
            )
            .await?;
        assert_json_is!(messages, []);
        assert_eq!(outputs.len(), 1);

        Ok(())
    }

    /// Test setting and getting of vars of different types
    #[tokio::test]
    async fn set_get_vars() -> Result<()> {
        let mut kernel = match skip_or_kernel().await {
            Ok(kernel) => kernel,
            Err(..) => return Ok(()),
        };

        kernel_micro::tests::set_get_strings(&mut kernel).await?;

        Ok(())
    }
}
