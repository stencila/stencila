use std::{
    path::Path,
    sync::{
        atomic::{AtomicU64, AtomicU8, Ordering},
        Arc, RwLock,
    },
};

use rhai::{Dynamic, Engine, Map, Scope};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        serde_json,
        tokio::{
            self,
            sync::{mpsc, watch},
        },
        tracing,
    },
    format::Format,
    schema::{ExecutionError, Node, NodeType, Null, Variable},
    Kernel, KernelAvailability, KernelForks, KernelInstance, KernelInterrupt, KernelKill,
    KernelSignal, KernelStatus, KernelTerminate,
};

/// A kernel for executing Rhai.
///
/// Rhai is an embedded scripting language and engine for Rust (https://rhai.rs/).
#[derive(Default)]
pub struct RhaiKernel {
    /// A counter of instances of this kernel
    instances: AtomicU64,
}

impl Kernel for RhaiKernel {
    fn id(&self) -> String {
        "rhai".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        KernelAvailability::Available
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Rhai]
    }

    fn supports_interrupt(&self) -> KernelInterrupt {
        KernelInterrupt::No
    }

    fn supports_terminate(&self) -> KernelTerminate {
        KernelTerminate::Yes
    }

    fn supports_kill(&self) -> KernelKill {
        KernelKill::No
    }

    fn supports_forks(&self) -> KernelForks {
        KernelForks::No
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        // Assign an id for the instance using the index, if necessary, to ensure it is unique
        let index = self.instances.fetch_add(1, Ordering::SeqCst);
        let id = if index == 0 {
            self.id()
        } else {
            format!("{}-{index}", self.id())
        };

        Ok(Box::new(RhaiKernelInstance::new(id)))
    }
}

pub struct RhaiKernelInstance<'lt> {
    /// The id of this instance
    id: String,

    /// The Rhai execution scope for this instance
    scope: Scope<'lt>,

    /// The Rhai execution engine for this instance
    engine: Engine,

    /// The status of this instance
    status: Arc<AtomicU8>,

    /// A channel sender for the status of this instance
    status_sender: watch::Sender<KernelStatus>,

    /// A channel sender for sending signals to the instance
    signal_sender: mpsc::Sender<KernelSignal>,
}

#[async_trait]
impl<'lt> KernelInstance for RhaiKernelInstance<'lt> {
    fn id(&self) -> String {
        self.id.clone()
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(self.get_status())
    }

    fn watcher(&self) -> Result<watch::Receiver<KernelStatus>> {
        Ok(self.status_sender.subscribe())
    }

    fn signaller(&self) -> Result<mpsc::Sender<KernelSignal>> {
        Ok(self.signal_sender.clone())
    }

    async fn start(&mut self, _directory: &Path) -> Result<()> {
        self.set_status(KernelStatus::Ready)
    }

    async fn stop(&mut self) -> Result<()> {
        self.set_status(KernelStatus::Stopped)
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        let status = self.get_status();
        if status != KernelStatus::Ready {
            bail!("Kernel `{}` is not ready; status is `{status}`", self.id())
        }

        self.set_status(KernelStatus::Busy)?;

        let outputs = Arc::new(RwLock::new(Vec::new()));

        let outputs_clone = outputs.clone();
        self.engine.on_print(move |value: &str| {
            let node = match serde_json::from_str(value) {
                Ok(node) => node,
                Err(..) => Node::String(value.to_string()),
            };
            if let Ok(mut guard) = outputs_clone.write() {
                guard.push(node)
            }
        });

        // TODO: if last 'line' is an expression use it as output
        let result = self.engine.run_with_scope(&mut self.scope, code);

        let outputs = outputs
            .read()
            .map_err(|error| eyre!("While reading outputs: {error}"))?
            .to_owned();

        self.set_status(KernelStatus::Ready)?;

        match result {
            Ok(..) => Ok((outputs, vec![])),
            Err(error) => Ok((vec![], vec![ExecutionError::new(error.to_string())])),
        }
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        let status = self.get_status();
        if status != KernelStatus::Ready {
            bail!("Kernel `{}` is not ready; status is `{status}`", self.id())
        }

        self.set_status(KernelStatus::Busy)?;

        let result = self
            .engine
            .eval_expression_with_scope::<Dynamic>(&mut self.scope, code);

        self.set_status(KernelStatus::Ready)?;

        match result {
            Ok(value) => Ok((vec![dynamic_to_node(value)?], vec![])),
            Err(error) => Ok((vec![], vec![ExecutionError::new(error.to_string())])),
        }
    }

    async fn list(&mut self) -> Result<Vec<Variable>> {
        let variables = self
            .scope
            .iter()
            .map(|(name, _, value)| {
                let name = name.to_string();

                let programming_language = Some("Rhai".to_string());
                let native_type = Some(value.type_name().to_string());

                let node_type = dynamic_to_node_type(value);

                Variable {
                    name,
                    programming_language,
                    native_type,
                    node_type,
                    ..Default::default()
                }
            })
            .collect();

        Ok(variables)
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        let Some(value) = self.scope.get_value::<Dynamic>(name) else {
            return Ok(None)
        };

        let node = dynamic_to_node(value)?;

        Ok(Some(node))
    }

    async fn set(&mut self, name: &str, node: &Node) -> Result<()> {
        let value = node_to_dynamic(node)?;

        self.scope.set_or_push(name, value);

        Ok(())
    }

    async fn remove(&mut self, name: &str) -> Result<()> {
        let _ = self.scope.remove::<()>(name);

        Ok(())
    }

    async fn fork(&mut self) -> Result<Box<dyn KernelInstance>> {
        // TODO
        todo!()
    }
}

impl<'lt> RhaiKernelInstance<'lt> {
    /// Create a new kernel instance
    fn new(id: String) -> Self {
        let scope = Scope::new();
        let engine = Engine::new();

        let status = Arc::new(AtomicU8::new(KernelStatus::Pending.into()));
        let (status_sender, mut status_receiver) = watch::channel(KernelStatus::Pending);

        // Start an async task to log status. This is useful for debugging but could be disabled
        // for release builds.
        let id_clone = id.clone();
        tokio::spawn(async move {
            while status_receiver.changed().await.is_ok() {
                let status = *status_receiver.borrow_and_update();
                tracing::trace!("Status of `{id_clone}` kernel changed: {status}")
            }
        });

        let (signal_sender, mut signal_receiver) = mpsc::channel(1);

        // Start a task to handle signals
        let status_clone = status.clone();
        tokio::spawn(async move {
            while let Some(kernel_signal) = signal_receiver.recv().await {
                if matches!(kernel_signal, KernelSignal::Terminate | KernelSignal::Kill) {
                    status_clone.store(KernelStatus::Stopped.into(), Ordering::SeqCst);
                }
            }
        });

        Self {
            id,
            scope,
            engine,
            status,
            status_sender,
            signal_sender,
        }
    }

    /// Get the status of the kernel
    fn get_status(&self) -> KernelStatus {
        self.status.load(Ordering::SeqCst).into()
    }

    /// Set the status of the kernel instance and notify watchers if there was a change
    ///
    /// Avoids overwriting of `Stopping` or `Stopped` status (which can happen when a
    /// `Terminate` signal is received while a task is executing)
    fn set_status(&mut self, status: KernelStatus) -> Result<()> {
        let previous: KernelStatus = self.status.swap(status.into(), Ordering::SeqCst).into();
        if previous >= KernelStatus::Stopping {
            self.status.store(previous.into(), Ordering::SeqCst);
            return Ok(());
        }

        self.status_sender.send_if_modified(|previous| {
            if status != *previous {
                *previous = status;
                true
            } else {
                false
            }
        });

        Ok(())
    }
}

/// Get the `NodeType` corresponding to a Rhai `Dynamic` type
fn dynamic_to_node_type(value: Dynamic) -> Option<String> {
    match value.type_name() {
        "()" => Some(NodeType::Null.to_string()),
        "bool" => Some(NodeType::Boolean.to_string()),
        "i64" => Some(NodeType::Integer.to_string()),
        "f64" => Some(NodeType::Number.to_string()),
        "string" | "char" => Some(NodeType::String.to_string()),
        "array" => Some(NodeType::Array.to_string()),
        _ => {
            if let Some(map) = value.try_cast::<Map>() {
                if let Some(typ) = map
                    .get("type")
                    .and_then(|value| value.clone().try_cast::<String>())
                {
                    Some(typ)
                } else {
                    Some(NodeType::Object.to_string())
                }
            } else {
                None
            }
        }
    }
}

/// Convert a Rhai `Dynamic` value to a Stencila `Node`
fn dynamic_to_node(value: Dynamic) -> Result<Node> {
    let node = match value.type_name() {
        "()" => Node::Null(Null),
        "bool" => Node::Boolean(value.cast::<bool>()),
        "i64" => Node::Integer(value.cast::<i64>()),
        "f64" => Node::Number(value.cast::<f64>()),
        "alloc::string::String" => Node::String(value.cast::<String>()),
        "char" => Node::String(String::from(value.cast::<char>())),
        _ => {
            // Resort to conversion using serde
            serde_json::from_value(serde_json::to_value(&value)?)?
        }
    };

    Ok(node)
}

/// Convert a Stencila `Node` to a Rhai `Dynamic` value
fn node_to_dynamic(node: &Node) -> Result<Dynamic> {
    Ok(match node {
        Node::Null(..) => ().into(),
        Node::Boolean(value) => (*value).into(),
        Node::Integer(value) => (*value).into(),
        Node::UnsignedInteger(value) => (*value as i64).into(),
        Node::Number(value) => (*value).into(),
        Node::String(value) => value.into(),
        _ => {
            // Resort to conversion using serde
            serde_json::from_value(serde_json::to_value(node)?)?
        }
    })
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{
        common::{
            eyre::Report,
            indexmap::IndexMap,
            tokio::{self, sync::mpsc},
            tracing,
        },
        schema::{Array, Node, Object, Paragraph, Primitive},
        KernelSignal, KernelStatus,
    };

    use super::*;

    /// Create and start a new kernel instance
    async fn start_kernel() -> Result<Option<Box<dyn KernelInstance>>> {
        let mut instance = RhaiKernel::default().create_instance()?;
        instance.start_here().await?;

        Ok(Some(instance))
    }

    /// Test watching status and sending signals
    ///
    /// Pro-tip! Use this to get logs for this test:
    ///
    /// ```sh
    /// RUST_LOG=trace cargo test -p kernel-node status_and_signals -- --nocapture
    /// ```
    ///
    /// Needs to use multiple threads because no `await`s in `execute()` method
    /// for signals and watches to run on.
    #[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
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
            kernel.execute("sleep(0.2)").await?;
            let status = kernel.status().await?;
            if status != KernelStatus::Ready {
                tracing::error!("Unexpected status in step {step}: {status}")
            }

            // Sleep with terminate
            let step = step_receiver.recv().await.unwrap();
            kernel.execute("sleep(0.2)").await?;
            let status = kernel.status().await?;
            if status != KernelStatus::Stopped {
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

            // Terminate during second sleep
            signaller.send(KernelSignal::Terminate).await?;
        }

        // Should have finished the task with correct status
        let status = task.await??;
        assert_eq!(status, KernelStatus::Stopped);

        Ok(())
    }

    /// Test evaluate tasks that just generate outputs of different types
    #[tokio::test]
    async fn outputs() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // A number
        let (outputs, messages) = kernel.evaluate("1.23").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Number(1.23)]);

        // A string in double quotes
        let (outputs, messages) = kernel.evaluate("\"Hello\"").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("Hello".to_string())]);

        // An array
        let (outputs, messages) = kernel.evaluate("[1,2,3]").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![Node::Array(Array::from([
                Primitive::Integer(1),
                Primitive::Integer(2),
                Primitive::Integer(3)
            ]))]
        );

        // An object
        let (outputs, messages) = kernel.evaluate(r#"#{a:1, b:2.3}"#).await?;
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
            .evaluate(r#"#{"type":"Paragraph", "content":[]}"#)
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Paragraph(Paragraph::new(vec![]))]);

        Ok(())
    }

    /// Test execute tasks that declare and use state within the kernel
    #[tokio::test]
    async fn execute_state() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        // Declare some variables
        let (outputs, messages) = kernel.execute("let a=1;\nlet b=2").await?;
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

        // List existing vars
        let initial = kernel.list().await?;
        assert_eq!(initial.len(), 0);

        // Set a var
        let var_name = "var1";
        let var_val = Node::String("Hello Rhai!".to_string());
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

    /// Test declaring variables with different types
    #[tokio::test]
    async fn var_types() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
                return Ok(())
            };

        kernel
            .execute(
                r#"
            let n = 1.23;
            let s = "str";
            let a = [1, 2, 3];
            let o = #{a:1, b:2.3};
        "#,
            )
            .await?;

        let vars = kernel.list().await?;

        let var = vars.iter().find(|var| var.name == "n").unwrap();
        assert_eq!(var.native_type.as_deref(), Some("f64"));
        assert_eq!(var.node_type.as_deref(), Some("Number"));
        assert_eq!(kernel.get("n").await?, Some(Node::Number(1.23)));

        let var = vars.iter().find(|var| var.name == "s").unwrap();
        assert_eq!(var.native_type.as_deref(), Some("string"));
        assert_eq!(var.node_type.as_deref(), Some("String"));
        assert!(matches!(kernel.get("s").await?, Some(Node::String(..))));

        let var = vars.iter().find(|var| var.name == "a").unwrap();
        assert_eq!(var.native_type.as_deref(), Some("array"));
        assert_eq!(var.node_type.as_deref(), Some("Array"));
        assert_eq!(
            kernel.get("a").await?,
            Some(Node::Array(Array(vec![
                Primitive::Integer(1),
                Primitive::Integer(2),
                Primitive::Integer(3)
            ])))
        );

        let var = vars.iter().find(|var| var.name == "o").unwrap();
        assert_eq!(var.native_type.as_deref(), Some("map"));
        assert_eq!(var.node_type.as_deref(), Some("Object"));
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
        assert_eq!(
            messages[0].error_message,
            "Syntax error: '#' is a reserved symbol (line 1, position 7)"
        );
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(
            messages[0].error_message,
            "Variable not found: foo (line 1, position 1)"
        );
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
    #[ignore]
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

    /// Test that `print`ed values are treated as outputs
    #[tokio::test]
    async fn printing() -> Result<()> {
        let Some(mut kernel) = start_kernel().await? else {
            return Ok(())
        };

        let (outputs, messages) = kernel
            .execute(
                r#"
print(true);
print(123);
print(4.56);
print("Hello")
"#,
            )
            .await?;

        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::Boolean(true),
                Node::Integer(123),
                Node::Number(4.56),
                Node::String("Hello".to_string())
            ]
        );

        Ok(())
    }
}
