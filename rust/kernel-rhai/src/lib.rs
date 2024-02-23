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
    schema::{
        ArrayHint, ExecutionMessage, Hint, MessageLevel, Node, NodeType, Null, ObjectHint,
        SoftwareApplication, SoftwareApplicationOptions, SoftwareSourceCode, StringHint, Unknown,
        Variable,
    },
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

    /// A counter of forks of this instance
    forks: AtomicU64,
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

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
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
            Err(error) => (
                vec![],
                vec![ExecutionMessage::new(
                    MessageLevel::Error,
                    error.to_string(),
                )],
            ),
        };

        if let Some(last) = last {
            outputs.push(dynamic_to_node(last)?);
        }

        self.set_status(KernelStatus::Ready)?;

        Ok((outputs, messages))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
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
                vec![ExecutionMessage::new(
                    MessageLevel::Error,
                    error.to_string(),
                )],
            )),
        };

        self.set_status(KernelStatus::Ready)?;

        result
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Rhai runtime info");

        Ok(SoftwareApplication {
            name: "Rhai".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                software_version: Some("1".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    async fn packages(&mut self) -> Result<Vec<SoftwareSourceCode>> {
        tracing::trace!("Getting Rhai packages");

        Ok(vec![])
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
                let (node_type, hint) = dynamic_to_type_hint(value);

                Variable {
                    name,
                    programming_language,
                    native_type,
                    node_type,
                    hint,
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
        tracing::trace!("Forking Rhai kernel instance");

        // Create fork id
        let id = format!(
            "{}-fork-{}",
            self.id,
            self.forks.fetch_add(1, Ordering::SeqCst)
        );

        // Create instance
        let mut fork = RhaiKernelInstance::new(id);

        // Clone variables into fork's scope
        for (name, ..) in self.scope.iter() {
            if let Some(value) = self.scope.get(name) {
                fork.scope.set_value(name, value.clone());
            }
        }

        Ok(Box::new(fork))
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
            forks: Default::default(),
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

/// Create a `NodeType` string and `Hint` for a Rhai `Dynamic` value
fn dynamic_to_type_hint(value: Dynamic) -> (Option<String>, Option<Hint>) {
    match value.type_name() {
        "()" => (Some(NodeType::Null.to_string()), None),
        "bool" => (
            Some(NodeType::Boolean.to_string()),
            Some(Hint::Boolean(value.as_bool().expect("should be bool"))),
        ),
        "i64" => (
            Some(NodeType::Integer.to_string()),
            Some(Hint::Integer(value.as_int().expect("should be int"))),
        ),
        "f64" => (
            Some(NodeType::Number.to_string()),
            Some(Hint::Number(value.as_float().expect("should be float"))),
        ),
        "char" => (
            Some(NodeType::String.to_string()),
            Some(Hint::StringHint(StringHint::new(1))),
        ),
        "string" => (
            Some(NodeType::String.to_string()),
            Some(Hint::StringHint(StringHint::new(
                value
                    .into_immutable_string()
                    .expect("should be string")
                    .chars()
                    .count() as i64,
            ))),
        ),
        "array" => (
            Some(NodeType::Array.to_string()),
            Some(Hint::ArrayHint(ArrayHint::new(
                value.into_array().expect("should be array").len() as i64,
            ))),
        ),
        _ => {
            if let Some(map) = value.try_cast::<Map>() {
                if let Some(typ) = map
                    .get("type")
                    .and_then(|value| value.clone().try_cast::<String>())
                {
                    (Some(typ), None)
                } else {
                    let length = map.len() as i64;
                    let keys = map.keys().map(|key| key.to_string()).collect();
                    let values = map
                        .into_values()
                        .map(|value| {
                            dynamic_to_type_hint(value)
                                .1
                                .unwrap_or(Hint::Unknown(Unknown::new()))
                        })
                        .collect();
                    (
                        Some(NodeType::Object.to_string()),
                        Some(Hint::ObjectHint(ObjectHint::new(length, keys, values))),
                    )
                }
            } else {
                (None, None)
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

    // Pro-tip! Use get logs for these tests use:
    //
    // ```sh
    // RUST_LOG=trace cargo test -p kernel-rhai -- --nocapture
    // ```

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

    /// Custom test for execution messages
    #[tokio::test]
    async fn messages() -> Result<()> {
        let Some(mut kernel) = start_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("bad ^ # syntax").await?;
        assert_eq!(
            messages[0].message,
            "Syntax error: '#' is a reserved symbol (line 1, position 7)"
        );
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(
            messages[0].message,
            "Variable not found: foo (line 1, position 1)"
        );
        assert_eq!(outputs, vec![]);

        Ok(())
    }

    /// Standard kernel test for getting runtime information
    #[test_log::test(tokio::test)]
    async fn info() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        let sw = kernel::tests::info(instance).await?;
        assert_eq!(sw.name, "Rhai");
        assert!(sw.options.software_version.is_some());
        assert!(sw.options.software_version.unwrap().starts_with('1'));

        Ok(())
    }

    /// Standard kernel test for listing installed packages
    #[test_log::test(tokio::test)]
    async fn packages() -> Result<()> {
        let Some(instance) = start_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        let pkgs = kernel::tests::packages(instance).await?;
        assert!(pkgs.is_empty());

        Ok(())
    }

    /// Standard kernel test for variable listing
    #[test_log::test(tokio::test)]
    async fn var_listing() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        kernel::tests::var_listing(
            instance,
            r#"
let nul = ();
let bool = true;
let int = 123;
let num = 1.23;
let str = "abc👍";
let arr = [1, 2, 3];
let obj = #{a:1, b:2.3};
let para = #{type:"Paragraph", content:[]};
"#,
            vec![
                Variable {
                    name: "nul".to_string(),
                    native_type: Some("()".to_string()),
                    node_type: Some("Null".to_string()),
                    programming_language: Some("Rhai".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "bool".to_string(),
                    native_type: Some("bool".to_string()),
                    node_type: Some("Boolean".to_string()),
                    hint: Some(Hint::Boolean(true)),
                    programming_language: Some("Rhai".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "int".to_string(),
                    native_type: Some("i64".to_string()),
                    node_type: Some("Integer".to_string()),
                    hint: Some(Hint::Integer(123)),
                    programming_language: Some("Rhai".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "num".to_string(),
                    native_type: Some("f64".to_string()),
                    node_type: Some("Number".to_string()),
                    hint: Some(Hint::Number(1.23)),
                    programming_language: Some("Rhai".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "str".to_string(),
                    native_type: Some("string".to_string()),
                    node_type: Some("String".to_string()),
                    hint: Some(Hint::StringHint(StringHint::new(4))),
                    programming_language: Some("Rhai".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "arr".to_string(),
                    native_type: Some("array".to_string()),
                    node_type: Some("Array".to_string()),
                    hint: Some(Hint::ArrayHint(ArrayHint::new(3))),
                    programming_language: Some("Rhai".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "obj".to_string(),
                    native_type: Some("map".to_string()),
                    node_type: Some("Object".to_string()),
                    hint: Some(Hint::ObjectHint(ObjectHint::new(
                        2,
                        vec!["a".to_string(), "b".to_string()],
                        vec![Hint::Integer(1), Hint::Number(2.3)],
                    ))),
                    programming_language: Some("Rhai".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "para".to_string(),
                    native_type: Some("map".to_string()),
                    node_type: Some("Paragraph".to_string()),
                    programming_language: Some("Rhai".to_string()),
                    ..Default::default()
                },
            ],
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

    /// Standard kernel test for forking
    #[test_log::test(tokio::test)]
    async fn forking() -> Result<()> {
        let Some(instance) = create_instance::<RhaiKernel>().await? else {
            return Ok(());
        };

        kernel::tests::forking(instance).await
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
}
