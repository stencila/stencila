use std::sync::atomic::{AtomicU64, Ordering};

use kernel_micro::{
    common::eyre::Result, format::Format, Kernel, KernelAvailability, KernelForks, KernelInstance,
    KernelInterrupt, KernelKill, Microkernel,
};

/// A kernel for executing Bash code locally
#[derive(Default)]
pub struct BashKernel {
    /// A counter of instances of this microkernel
    instances: AtomicU64,
}

impl Kernel for BashKernel {
    fn id(&self) -> String {
        "bash-micro".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        self.microkernel_availability()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Bash, Format::Shell]
    }

    fn supports_interrupt(&self) -> KernelInterrupt {
        self.microkernel_supports_interrupt()
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

impl Microkernel for BashKernel {
    fn executable_name(&self) -> String {
        "bash".to_string()
    }

    fn microkernel_script(&self) -> String {
        include_str!("kernel.bash").to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use common_dev::pretty_assertions::assert_eq;
    use kernel_micro::{
        common::{
            eyre::{bail, Report},
            tokio::{self, sync::mpsc},
            tracing,
        },
        schema::{Array, Node, Object, Paragraph, Primitive},
        KernelSignal, KernelStatus,
    };

    use super::*;

    /// Create and start a new kernel instance if Bash is available
    async fn start_kernel() -> Result<Option<Box<dyn KernelInstance>>> {
        let kernel = BashKernel::default();
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
    /// RUST_LOG=trace cargo test -p kernel-bash status_and_signals -- --nocapture
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
            kernel.execute("sleep 0.25").await?;
            let status = kernel.status().await?;
            if status != KernelStatus::Ready {
                tracing::error!("Unexpected status in step {step}: {status}")
            }

            // Sleep with interrupt
            let step = step_receiver.recv().await.unwrap();
            kernel.execute("sleep 100").await?;
            let status = kernel.status().await?;
            if status != KernelStatus::Ready {
                tracing::error!("Unexpected status in step {step}: {status}")
            }

            // Sleep with kill
            let step = step_receiver.recv().await.unwrap();
            kernel.execute("sleep 100").await?;
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

            // Should be busy during second sleep
            watcher.changed().await?;
            assert_eq!(*watcher.borrow_and_update(), KernelStatus::Busy);

            // Kill during second sleep (if this fails then the test would keep running for 100 seconds)
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

        // Print a string
        let (outputs, messages) = kernel.execute("echo 'Hello'").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("Hello\n".to_string())]);

        // Print a string in double quotes
        let (outputs, messages) = kernel.execute("printf \"Hello\"").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("Hello".to_string())]);

        // Print a number
        let (outputs, messages) = kernel.execute("echo '1.23'").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Number(1.23)]);

        // Print an array
        let (outputs, messages) = kernel.execute("echo '[1,2,3]'").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![Node::Array(Array::from([
                Primitive::Integer(1),
                Primitive::Integer(2),
                Primitive::Integer(3)
            ]))]
        );

        // Print an object
        let (outputs, messages) = kernel.execute(r#"echo '{"a":1, "b":2.3}'"#).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![Node::Object(Object::from([
                ("a", Primitive::Integer(1)),
                ("b", Primitive::Number(2.3))
            ]))]
        );

        // Print a content node type
        let (outputs, messages) = kernel
            .execute(r#"echo '{"type":"Paragraph", "content":[]}'"#)
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
        let (outputs, messages) = kernel.execute("a=1\nb=2").await?;
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
        assert_eq!(
            initial
                .iter()
                .filter(|var| var.name == "PATH" && var.node_type.as_deref() == Some("String"))
                .count(),
            1
        );

        // Get a var
        assert_eq!(
            kernel.get("PATH").await?,
            env::var("PATH").ok().map(Node::String)
        );

        // Set a var
        let var_name = "MYVAR";
        let var_val = Node::String("VAL".to_string());
        kernel.set(var_name, &var_val).await?;

        // Get the var
        assert_eq!(kernel.get(var_name).await?, Some(var_val));

        // Remove the var
        kernel.remove(var_name).await?;
        assert_eq!(kernel.get(var_name).await?, None);

        Ok(())
    }

    /// Test declaring Bash variables with different types
    #[tokio::test]
    async fn var_types() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
                return Ok(())
            };

        kernel
            .execute(
                r#"
            declare s="str"
            declare -a a=(1 2 3)
            declare -A o=(["key1"]="value1" ["key2"]="value2")
        "#,
            )
            .await?;

        let vars = kernel.list().await?;

        let var = vars.iter().find(|var| var.name == "s").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("String"));
        assert_eq!(var.native_type.as_deref(), Some("string"));
        assert!(matches!(kernel.get("s").await?, Some(Node::String(..))));

        let var = vars.iter().find(|var| var.name == "a").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("Array"));
        assert_eq!(var.native_type.as_deref(), Some("array"));
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
        assert_eq!(var.native_type.as_deref(), Some("associative array"));

        Ok(())
    }

    /// Test execute tasks that intentionally generate error messages
    #[tokio::test]
    async fn messages() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("if").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(outputs, vec![]);

        Ok(())
    }

    /// Test forking of microkernel
    ///
    /// Pro-tip! Use this to get logs for this test:
    ///
    /// ```sh
    /// RUST_LOG=trace cargo test -p kernel-bash forks -- --nocapture
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

    /// Test execution tasks that involve additional escaping
    #[tokio::test]
    async fn escaping() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // Test escaping of percent signs in commands
        let (outputs, messages) = kernel.execute("date +%s").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs.len(), 1);

        match outputs.first() {
            Some(Node::Integer(timestamp)) => assert!(*timestamp > 1600000000),
            _ => bail!("Expected an integer output"),
        }

        Ok(())
    }
}
