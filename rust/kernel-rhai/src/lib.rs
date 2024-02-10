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
        itertools::Itertools,
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
        tracing::trace!("Executing Rhai code");

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

        let mut lines = code.lines().collect_vec();
        let (rest, last) = match lines.last().map(|line| {
            self.engine
                .eval_expression_with_scope::<Dynamic>(&mut self.scope, line)
        }) {
            Some(Ok(output)) => {
                lines.pop();
                (lines.join("\n"), Some(output))
            }
            _ => (lines.join("\n"), None),
        };

        let (mut outputs, messages) = match self.engine.run_with_scope(&mut self.scope, &rest) {
            Ok(..) => (
                outputs
                    .read()
                    .map_err(|error| eyre!("While reading outputs: {error}"))?
                    .to_owned(),
                vec![],
            ),
            Err(error) => (vec![], vec![ExecutionError::new(error.to_string())]),
        };

        if let Some(last) = last {
            outputs.push(dynamic_to_node(last)?);
        }

        self.set_status(KernelStatus::Ready)?;

        Ok((outputs, messages))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionError>)> {
        tracing::trace!("Evaluating Rhai code");

        let status = self.get_status();
        if status != KernelStatus::Ready {
            bail!("Kernel `{}` is not ready; status is `{status}`", self.id())
        }

        if code.trim().is_empty() {
            return Ok((Node::Null(Null), vec![]));
        }

        self.set_status(KernelStatus::Busy)?;

        let result = match self
            .engine
            .eval_expression_with_scope::<Dynamic>(&mut self.scope, code)
        {
            Ok(value) => Ok((dynamic_to_node(value)?, vec![])),
            Err(error) => Ok((
                Node::Null(Null),
                vec![ExecutionError::new(error.to_string())],
            )),
        };

        self.set_status(KernelStatus::Ready)?;

        result
    }

    async fn list(&mut self) -> Result<Vec<Variable>> {
        tracing::trace!("Listing Rhai variables");

        self.set_status(KernelStatus::Busy)?;

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

        self.set_status(KernelStatus::Ready)?;

        Ok(variables)
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        tracing::trace!("Getting Rhai variable");

        self.set_status(KernelStatus::Busy)?;

        let node = match self.scope.get_value::<Dynamic>(name) {
            Some(value) => Some(dynamic_to_node(value)?),
            None => None,
        };

        self.set_status(KernelStatus::Ready)?;

        Ok(node)
    }

    async fn set(&mut self, name: &str, node: &Node) -> Result<()> {
        tracing::trace!("Setting Rhai variable");

        self.set_status(KernelStatus::Busy)?;

        self.scope.set_or_push(name, node_to_dynamic(node)?);

        self.set_status(KernelStatus::Ready)
    }

    async fn remove(&mut self, name: &str) -> Result<()> {
        tracing::trace!("Removing Rhai variable");

        self.set_status(KernelStatus::Busy)?;

        let _ = self.scope.remove::<()>(name);

        self.set_status(KernelStatus::Ready)
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
        let (status_sender, ..) = watch::channel(KernelStatus::Pending);

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
                tracing::trace!(
                    "Status of `{}` kernel changed from `{previous}` to `{status}`",
                    self.id()
                );
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
        common::{indexmap::IndexMap, tokio},
        schema::{Array, Node, Object, Primitive},
        tests::{create_instance, start_instance},
    };

    use super::*;

    /// Standard kernel test for execution of code
    #[test_log::test(tokio::test)]
    async fn execution() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        kernel::tests::execution(
            instance,
            vec![
                // Empty code: no outputs
                ("", vec![], vec![]),
                (" ", vec![], vec![]),
                ("\n\n", vec![], vec![]),
                // Only an expression: one output
                (
                    "
1 + 1",
                    vec![Node::Integer(2)],
                    vec![],
                ),
                // Prints and an expression: multiple, separate outputs
                (
                    "
print(1);
print(2);
1 + 2",
                    vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)],
                    vec![],
                ),
                // Variables set in one chunk are available in the next
                (
                    "
let a = 1;
let b = 2;",
                    vec![],
                    vec![],
                ),
                (
                    "
print(a);
b",
                    vec![Node::Integer(1), Node::Integer(2)],
                    vec![],
                ),
            ],
        )
        .await
    }

    /// Standard kernel test for evaluation of expressions
    #[test_log::test(tokio::test)]
    async fn evaluation() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        kernel::tests::evaluation(
            instance,
            vec![
                ("1 + 1", Node::Integer(2), None),
                ("2.0 * 2.2", Node::Number(4.4), None),
                ("16 ** 0.5", Node::Number(4.0), None),
                (r#"'a' + "bc""#, Node::String("abc".to_string()), None),
                (r#""ABC".to_lower()"#, Node::String("abc".to_string()), None),
                (
                    "[1, 2] + [3]",
                    Node::Array(Array(vec![
                        Primitive::Integer(1),
                        Primitive::Integer(2),
                        Primitive::Integer(3),
                    ])),
                    None,
                ),
                (
                    "#{a:1, b:2.3}",
                    Node::Object(Object(IndexMap::from([
                        (String::from("a"), Primitive::Integer(1)),
                        (String::from("b"), Primitive::Number(2.3)),
                    ]))),
                    None,
                ),
                ("", Node::Null(Null), None),
                (
                    "@",
                    Node::Null(Null),
                    Some("Syntax error: '@' is a reserved symbol (line 1, position 1)"),
                ),
                (
                    "foo",
                    Node::Null(Null),
                    Some("Variable not found: foo (line 1, position 1)"),
                ),
            ],
        )
        .await
    }

    /// Standard kernel test for printing nodes
    #[test_log::test(tokio::test)]
    async fn printing() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        kernel::tests::printing(
            instance,
            r#"print("str");"#,
            r#"print("str1"); print("str2");"#,
            r#"
                print("null");
                print(true);
                print(1);
                print(2.3);
                print("str");
                print([1, 2.3, "str"]);
                print(#{a:1, b:2.3, c:"str"}.to_json());
            "#,
            r#"print(#{type:"Paragraph", content:[]}.to_json());"#,
        )
        .await
    }

    /// Standard kernel test for variable management
    #[test_log::test(tokio::test)]
    async fn var_management() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        kernel::tests::var_management(instance).await
    }

    /// Standard kernel test for signals
    ///
    /// Needs to use multiple threads because no `await`s in `execute()` method
    /// for signals and watches to run on.
    #[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
    async fn signals() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        kernel::tests::signals(
            instance,
            "
sleep(0.1);
let value = 1;
print(value);",
            None,
            Some(
                "
sleep(0.1)",
            ),
            None,
        )
        .await
    }

    /// Standard kernel test for stopping
    #[test_log::test(tokio::test)]
    async fn stop() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        kernel::tests::stop(instance).await
    }

    /// Test declaring variables with different types
    #[tokio::test]
    async fn var_types() -> Result<()> {
        let Some(mut kernel) = start_instance::<RhaiKernel>().await? else {
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
        let Some(mut kernel) = start_instance::<RhaiKernel>().await? else {
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
        let Some(mut kernel) = start_instance::<RhaiKernel>().await? else {
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
}
