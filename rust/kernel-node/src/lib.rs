use std::sync::atomic::{AtomicU64, Ordering};

use kernel_micro::{
    common::eyre::Result, format::Format, Kernel, KernelAvailability, KernelForks, KernelInstance,
    KernelInterrupt, KernelKill, KernelTerminate, Microkernel,
};

/// A kernel for executing JavaScript code in Node.js
#[derive(Default)]
pub struct NodeKernel {
    /// A counter of instances of this microkernel
    instances: AtomicU64,
}

impl Kernel for NodeKernel {
    fn id(&self) -> String {
        "node".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        self.microkernel_availability()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::JavaScript]
    }

    fn supports_interrupt(&self) -> KernelInterrupt {
        self.microkernel_supports_interrupt()
    }

    fn supports_terminate(&self) -> KernelTerminate {
        self.microkernel_supports_terminate()
    }

    fn supports_kill(&self) -> KernelKill {
        self.microkernel_supports_kill()
    }

    fn supports_forks(&self) -> KernelForks {
        self.microkernel_supports_forks()
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        self.microkernel_create_instance(self.instances.fetch_add(1, Ordering::SeqCst))
    }
}

impl Microkernel for NodeKernel {
    fn executable_name(&self) -> String {
        "node".to_string()
    }

    fn microkernel_script(&self) -> String {
        include_str!("kernel.js").to_string()
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel_micro::{
        common::{
            eyre::Report,
            indexmap::IndexMap,
            tokio::{self, sync::mpsc},
            tracing,
        },
        schema::{Array, ExecutionError, Node, Object, Paragraph, Primitive},
        KernelSignal, KernelStatus,
    };

    use super::*;

    /// Create and start a new kernel instance if Node.js is available
    async fn start_kernel() -> Result<Option<Box<dyn KernelInstance>>> {
        let kernel = NodeKernel::default();
        match kernel.availability() {
            KernelAvailability::Available => {
                let mut instance = kernel.create_instance()?;
                instance.start_here().await?;
                Ok(Some(instance))
            }
            _ => Ok(None),
        }
    }

    /// Test watching status and sending signals
    ///
    /// Pro-tip! Use this to get logs for this test:
    ///
    /// ```sh
    /// RUST_LOG=trace cargo test -p kernel-node status_and_signals -- --nocapture
    /// ```
    #[test_log::test(tokio::test)]
    async fn status_and_signals() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        let mut watcher = kernel.watcher()?;
        let signaller = kernel.signaller()?;

        // Should be ready because already started
        assert_eq!(kernel.status().await?, KernelStatus::Ready);
        assert_eq!(*watcher.borrow_and_update(), KernelStatus::Ready);

        // Move the kernel into a task so we can asynchronously do things in it
        // The "step" channel helps coordinate with this thread
        let (step_sender, mut step_receiver) = mpsc::channel::<u8>(1);
        let task = tokio::spawn(async move {
            // Short sleep
            let step = step_receiver.recv().await.unwrap();
            kernel
                .execute(
                    "
                // Crude sleep function which can be called at top level without using await
                function sleep(milliseconds) {
                    const startTime = new Date().getTime();
                    let currentTime = null;
                
                    do {
                        currentTime = new Date().getTime();
                    } while (currentTime - startTime < milliseconds);
                }
                sleep(250)
            ",
                )
                .await?;
            let status = kernel.status().await?;
            if status != KernelStatus::Ready {
                tracing::error!("Unexpected status in step {step}: {status}")
            }

            // Sleep with interrupt
            let step = step_receiver.recv().await.unwrap();
            kernel.execute("sleep(100000)").await?;
            let status = kernel.status().await?;
            if status != KernelStatus::Ready {
                tracing::error!("Unexpected status in step {step}: {status}")
            }

            // Sleep with kill
            let step = step_receiver.recv().await.unwrap();
            kernel.execute("sleep(100000)").await?;
            let status = kernel.status().await?;
            if status != KernelStatus::Failed {
                tracing::error!("Unexpected status in step {step}: {status}")
            }

            Ok::<KernelStatus, Report>(status)
        });

        {
            step_sender.send(1).await?;

            // Should be busy during first sleep
            watcher.changed().await?;
            assert_eq!(*watcher.borrow_and_update(), KernelStatus::Busy);

            // Should be ready after first sleep
            watcher.changed().await?;
            assert_eq!(*watcher.borrow_and_update(), KernelStatus::Ready);
        }
        {
            step_sender.send(2).await?;

            // Should be busy during second sleep
            watcher.changed().await?;
            assert_eq!(*watcher.borrow_and_update(), KernelStatus::Busy);

            // Interrupt during third sleep (if this fails then the test would keep running for 100 seconds)
            signaller.send(KernelSignal::Interrupt).await?;

            // Should be ready after interrupt
            watcher.changed().await?;
            assert_eq!(*watcher.borrow_and_update(), KernelStatus::Ready);
        }
        {
            step_sender.send(3).await?;

            // Should be busy during third sleep
            watcher.changed().await?;
            assert_eq!(*watcher.borrow_and_update(), KernelStatus::Busy);

            // Kill during third sleep (if this fails then the test would keep running for 100 seconds)
            signaller.send(KernelSignal::Kill).await?;
        }

        // Should have finished the task with correct status
        let status = task.await??;
        assert_eq!(status, KernelStatus::Failed);

        Ok(())
    }

    /// Test execute tasks that just generate outputs of different types
    #[tokio::test]
    async fn execute_outputs() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // A string
        let (outputs, messages) = kernel.execute("'Hello'").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("Hello".to_string())]);

        // A string in double quotes
        let (outputs, messages) = kernel.execute("\"Hello\"").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("Hello".to_string())]);

        // A number
        let (outputs, messages) = kernel.execute("1.23").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Number(1.23)]);

        // An array
        let (outputs, messages) = kernel.execute("[1,2,3]").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![Node::Array(Array::from([
                Primitive::Integer(1),
                Primitive::Integer(2),
                Primitive::Integer(3)
            ]))]
        );

        // An object (needs to be parenthesized)
        let (outputs, messages) = kernel.execute(r#"({a:1, b:2.3})"#).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![Node::Object(Object::from([
                ("a", Primitive::Integer(1)),
                ("b", Primitive::Number(2.3))
            ]))]
        );

        // A content node type
        let (outputs, messages) = kernel
            .execute(r#"({"type":"Paragraph", "content":[]})"#)
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Paragraph(Paragraph::new(vec![]))]);

        Ok(())
    }

    /// Test execute tasks that set and use state within the kernel
    #[tokio::test]
    async fn execute_state() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // Set some variables
        let (outputs, messages) = kernel.execute("const a=1\nconst b=2").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![]);

        // Evaluate an expression
        let (outputs, messages) = kernel.evaluate("a + b").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(3)]);

        Ok(())
    }

    /// Test evaluate tasks
    #[tokio::test]
    async fn evaluate() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
                return Ok(())
            };

        let (outputs, messages) = kernel.evaluate("1 + 2").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(3)]);

        Ok(())
    }

    /// Test list, set and get tasks
    #[tokio::test]
    async fn vars() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
                return Ok(())
            };

        // List existing env vars
        let initial = kernel.list().await?;
        assert_eq!(initial.len(), 1); // Just the "console"

        // Set a var
        let var_name = "myVar";
        let var_val = Node::String("Hello Node.js!".to_string());
        kernel.set(var_name, &var_val).await?;
        assert_eq!(kernel.list().await?.len(), initial.len() + 1);

        // Get the var
        assert_eq!(kernel.get(var_name).await?, Some(var_val));

        // Remove the var
        kernel.remove(var_name).await?;
        assert_eq!(kernel.get(var_name).await?, None);
        assert_eq!(kernel.list().await?.len(), initial.len());

        Ok(())
    }

    /// Test declaring JavaScript variables with different types
    #[tokio::test]
    async fn var_types() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
                return Ok(())
            };

        kernel
            .execute(
                r#"
            var n = 1.23
            var s = "str"
            var a = [1, 2, 3]
            var o = {a:1, b:2.3}
        "#,
            )
            .await?;

        let vars = kernel.list().await?;

        let var = vars.iter().find(|var| var.name == "n").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("Number"));
        assert_eq!(var.native_type.as_deref(), Some("number"));
        assert_eq!(kernel.get("n").await?, Some(Node::Number(1.23)));

        let var = vars.iter().find(|var| var.name == "s").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("String"));
        assert_eq!(var.native_type.as_deref(), Some("string"));
        assert!(matches!(kernel.get("s").await?, Some(Node::String(..))));

        let var = vars.iter().find(|var| var.name == "a").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("Array"));
        assert_eq!(var.native_type.as_deref(), Some("Array"));
        assert_eq!(
            kernel.get("a").await?,
            Some(Node::Array(Array(vec![
                Primitive::Integer(1),
                Primitive::Integer(2),
                Primitive::Integer(3)
            ])))
        );

        let var = vars.iter().find(|var| var.name == "o").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("Object"));
        assert_eq!(var.native_type.as_deref(), Some("object"));
        assert_eq!(
            kernel.get("o").await?,
            Some(Node::Object(Object(IndexMap::from([
                (String::from("a"), Primitive::Integer(1),),
                (String::from("b"), Primitive::Number(2.3))
            ]))))
        );

        Ok(())
    }

    /// Test execute tasks that intentionally generate error messages
    #[tokio::test]
    async fn messages() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("bad ^ # syntax").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("SyntaxError"));
        assert_eq!(messages[0].error_message, "Invalid or unexpected token");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("ReferenceError"));
        assert_eq!(messages[0].error_message, "foo is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

        Ok(())
    }

    /// Test forking of microkernel
    ///
    /// Pro-tip! Use this to get logs for this test:
    ///
    /// ```sh
    /// RUST_LOG=trace cargo test -p kernel-node forks -- --nocapture
    /// ```
    #[test_log::test(tokio::test)]
    async fn forks() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // Set variables in the kernel
        kernel.set("var1", &Node::Integer(123)).await?;
        kernel.set("var2", &Node::Number(4.56)).await?;
        kernel
            .set("var3", &Node::String("Hello world".to_string()))
            .await?;

        // Create a fork and check that the variables are available in it
        let mut fork = kernel.fork().await?;
        assert_eq!(fork.get("var1").await?, Some(Node::Integer(123)));
        assert_eq!(fork.get("var2").await?, Some(Node::Number(4.56)));
        assert_eq!(
            fork.get("var3").await?,
            Some(Node::String("Hello world".to_string()))
        );

        // Change variables in fork and check that they are unchanged in main kernel
        fork.set("var1", &Node::Integer(321)).await?;
        fork.remove("var2").await?;
        fork.execute("var3 = 'Hello from fork'").await?;
        assert_eq!(kernel.get("var1").await?, Some(Node::Integer(123)));
        assert_eq!(kernel.get("var2").await?, Some(Node::Number(4.56)));
        assert_eq!(
            kernel.get("var3").await?,
            Some(Node::String("Hello world".to_string()))
        );

        Ok(())
    }

    /// Test that `console.log` arguments are treated as separate outputs
    #[tokio::test]
    async fn console_log() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        let (outputs, messages) = kernel.execute("console.log(1)").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(1)]);

        let (outputs, messages) = kernel.execute("console.log(1, 2, 3)").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)]
        );

        let (outputs, messages) = kernel.execute("console.log([1, 2, 3], 4, 'str')").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::Array(Array(vec![
                    Primitive::Integer(1),
                    Primitive::Integer(2),
                    Primitive::Integer(3)
                ])),
                Node::Integer(4),
                Node::String("str".to_string())
            ]
        );

        Ok(())
    }

    /// Test that `console.debug`, `console.warn` etc are treated as messages
    /// separate from `console.log` outputs
    #[tokio::test]
    async fn console_messages() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        let (outputs, messages) = kernel
            .execute(
                r#"
console.log(1)
console.debug("Debug message")
console.log(2)
console.info("Info message")
console.log(3)
console.warn("Warning message")
console.log(4)
console.error("Error message")
5
"#,
            )
            .await?;

        assert_eq!(
            messages,
            vec![
                ExecutionError {
                    error_type: Some("Debug".to_string()),
                    error_message: "Debug message".to_string(),
                    ..Default::default()
                },
                ExecutionError {
                    error_type: Some("Info".to_string()),
                    error_message: "Info message".to_string(),
                    ..Default::default()
                },
                ExecutionError {
                    error_type: Some("Warning".to_string()),
                    error_message: "Warning message".to_string(),
                    ..Default::default()
                },
                ExecutionError {
                    error_type: Some("Error".to_string()),
                    error_message: "Error message".to_string(),
                    ..Default::default()
                }
            ]
        );
        assert_eq!(
            outputs,
            vec![
                Node::Integer(1),
                Node::Integer(2),
                Node::Integer(3),
                Node::Integer(4),
                Node::Integer(5)
            ]
        );

        Ok(())
    }

    /// Test re-declarations of variables
    #[tokio::test]
    async fn redeclarations() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // A variable declared with `var`

        let (outputs, messages) = kernel.execute("var a = 1\na").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(1));

        let (outputs, messages) = kernel.execute("var a = 2\na").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(2));

        let (outputs, messages) = kernel.execute("let a = 3\na").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(3));

        let (outputs, messages) = kernel.execute("const a = 4\na").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(4));

        // A variable declared with `let`

        let (outputs, messages) = kernel.execute("let b = 1\nb").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(1));

        let (outputs, messages) = kernel.execute("let b = 2\nb").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(2));

        let (outputs, messages) = kernel.execute("b = 3\nb").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(3));

        // A variable declared with `const`

        let (outputs, messages) = kernel.execute("const c = 1\nc").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(1));

        let (.., messages) = kernel.execute("const c = 2\nc").await?;
        assert_eq!(
            messages[0].error_message,
            "Assignment to constant variable."
        );

        let (.., messages) = kernel.execute("c = 3\nc").await?;
        assert_eq!(
            messages[0].error_message,
            "Assignment to constant variable."
        );

        Ok(())
    }
}
