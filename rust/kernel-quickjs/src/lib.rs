#![allow(non_snake_case)]

use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{AtomicU64, AtomicU8, Ordering},
        Arc,
    },
};

use rquickjs::{
    class::Trace, function::Rest, Array as JsArray, AsyncContext, AsyncRuntime, BigInt, Ctx, Error,
    Object as JsObject, String as JsString, Value,
};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        indexmap::IndexMap,
        serde_json,
        tokio::{
            self,
            sync::{mpsc, watch},
        },
        tracing,
    },
    format::Format,
    schema::{
        Array, ArrayHint, ExecutionMessage, Hint, MessageLevel, Node, NodeType, Null, Object,
        ObjectHint, Primitive, SoftwareApplication, SoftwareApplicationOptions, SoftwareSourceCode,
        StringHint, Variable,
    },
    Kernel, KernelForks, KernelInstance, KernelSignal, KernelStatus, KernelTerminate,
};

/// A kernel for executing JavaScript using the QuickJS engine.
#[derive(Default)]
pub struct QuickJsKernel {
    /// A counter of instances of this kernel
    instances: AtomicU64,
}

impl Kernel for QuickJsKernel {
    fn name(&self) -> String {
        "quickjs".to_string()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::JavaScript]
    }

    fn supports_forks(&self) -> KernelForks {
        KernelForks::Yes
    }

    fn supports_terminate(&self) -> KernelTerminate {
        KernelTerminate::Yes
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        // Assign an id for the instance using the index, if necessary, to ensure it is unique
        let index = self.instances.fetch_add(1, Ordering::SeqCst);
        let id = if index == 0 {
            self.name()
        } else {
            format!("{}-{index}", self.name())
        };

        Ok(Box::new(QuickJsKernelInstance::new(id)))
    }
}

pub struct QuickJsKernelInstance {
    /// The id of this instance
    id: String,

    /// The QuickJs runtime context for this instance
    context: Option<AsyncContext>,

    /// The QuickJs runtime for this instance
    runtime: Option<AsyncRuntime>,

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
impl KernelInstance for QuickJsKernelInstance {
    fn name(&self) -> String {
        self.id.clone()
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(self.get_status())
    }

    fn status_watcher(&self) -> Result<watch::Receiver<KernelStatus>> {
        Ok(self.status_sender.subscribe())
    }

    fn signal_sender(&self) -> Result<mpsc::Sender<KernelSignal>> {
        Ok(self.signal_sender.clone())
    }

    async fn start(&mut self, _directory: &Path) -> Result<()> {
        tracing::trace!("Starting QuickJS kernel instance");

        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;

        self.runtime = Some(runtime);
        self.context = Some(context);

        self.set_status(KernelStatus::Ready)
    }

    async fn stop(&mut self) -> Result<()> {
        tracing::trace!("Stopping QuickJS kernel instance");

        self.set_status(KernelStatus::Stopped)
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing QuickJS code");

        let status = self.get_status();
        if status != KernelStatus::Ready {
            bail!(
                "Kernel `{}` is not ready; status is `{status}`",
                self.name()
            )
        }

        self.run_code(code).await
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        tracing::trace!("Evaluating QuickJS code");

        let status = self.get_status();
        if status != KernelStatus::Ready {
            bail!(
                "Kernel `{}` is not ready; status is `{status}`",
                self.name()
            )
        }

        let (mut outputs, messages) = self.run_code(code).await?;
        let output = if outputs.is_empty() {
            Node::Null(Null)
        } else {
            outputs.swap_remove(0)
        };

        Ok((output, messages))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting QuickJS runtime info");

        Ok(SoftwareApplication {
            name: "QuickJS".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                software_version: Some("1".to_string()),
                operating_system: Some(std::env::consts::OS.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    async fn packages(&mut self) -> Result<Vec<SoftwareSourceCode>> {
        tracing::trace!("Getting QuickJS packages");

        Ok(vec![])
    }

    async fn list(&mut self) -> Result<Vec<Variable>> {
        tracing::trace!("Listing QuickJS variables");

        self.set_status(KernelStatus::Busy)?;

        let variables = self
            .get_context()?
            .with(|ctx| {
                ctx.globals()
                    .into_iter()
                    .flatten()
                    .flat_map(|(name, value)| {
                        let Ok(name) = name.to_string() else {
                            return None;
                        };

                        let programming_language = Some("JavaScript".to_string());
                        let native_type = Some(
                            match value.type_name() {
                                "bool" => "boolean",
                                "big_int" => "bigint",
                                "float" => "number",
                                name => name,
                            }
                            .to_string(),
                        );
                        let (node_type, hint) = value_to_type_hint(value);

                        Some(Variable {
                            name,
                            programming_language,
                            native_type,
                            node_type,
                            hint,
                            ..Default::default()
                        })
                    })
                    .collect()
            })
            .await;

        self.set_status(KernelStatus::Ready)?;

        Ok(variables)
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        tracing::trace!("Getting QuickJS variable");

        self.set_status(KernelStatus::Busy)?;

        let node = self
            .get_context()?
            .with(|ctx| match ctx.globals().get::<_, Value>(name) {
                Ok(value) => {
                    if value.is_undefined() {
                        Ok(None)
                    } else {
                        value_to_node(ctx, value).map(Some)
                    }
                }
                Err(..) => Ok(None),
            })
            .await?;

        self.set_status(KernelStatus::Ready)?;

        Ok(node)
    }

    async fn set(&mut self, name: &str, node: &Node) -> Result<()> {
        tracing::trace!("Setting QuickJS variable");

        self.set_status(KernelStatus::Busy)?;

        self.get_context()?
            .with(|ctx| -> Result<(), Error> {
                let globals = ctx.globals();
                let value = node_to_value(ctx, node);
                globals.set(name, value)
            })
            .await?;

        self.set_status(KernelStatus::Ready)
    }

    async fn remove(&mut self, name: &str) -> Result<()> {
        tracing::trace!("Removing QuickJS variable");

        self.set_status(KernelStatus::Busy)?;

        self.get_context()?
            .with(|ctx| ctx.globals().remove(name))
            .await?;

        self.set_status(KernelStatus::Ready)
    }

    async fn fork(&mut self) -> Result<Box<dyn KernelInstance>> {
        tracing::trace!("Forking QuickJS kernel instance");

        // Create fork id
        let id = format!(
            "{}-fork-{}",
            self.id,
            self.forks.fetch_add(1, Ordering::SeqCst)
        );

        // Create instance
        let mut fork = QuickJsKernelInstance::new(id);

        // Clone global variables into fork
        // Currently works by converting variables to/from Stencila nodes.
        // A more efficient way to do this may be possible.
        let vars = self
            .get_context()?
            .with(|ctx| -> HashMap<String, Node> {
                let globals = ctx.globals();

                let mut vars = HashMap::new();
                for name in globals.keys::<String>().flatten() {
                    let Ok(value): Result<Value, Error> = globals.get(&name) else {
                        continue;
                    };

                    if value.is_constructor() || value.is_function() {
                        continue;
                    }

                    if let Ok(node) = value_to_node(ctx.clone(), value) {
                        vars.insert(name, node);
                    }
                }

                vars
            })
            .await;
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;
        context
            .with(|ctx| {
                let globals = ctx.globals();
                for (name, node) in vars {
                    globals.set(name, node_to_value(ctx.clone(), &node)).ok();
                }
            })
            .await;

        fork.runtime = Some(runtime);
        fork.context = Some(context);

        Ok(Box::new(fork))
    }
}

/// Convert a QuickJS value to a Stencila Node
///
/// Checks if the value is an object with a "type" property and if so attempts to
/// use JSON and serde to do the conversion. Otherwise, converts as a primitive.
fn value_to_node<'js>(ctx: Ctx<'js>, value: Value<'js>) -> Result<Node, Error> {
    if let Some(object) = value.as_object() {
        if object.get::<_, String>("type").is_ok() {
            if let Some(node) = ctx
                .json_stringify(value.clone())
                .ok()
                .flatten()
                .and_then(|json| json.to_string().ok())
                .and_then(|json| serde_json::from_str(&json).ok())
            {
                return Ok(node);
            };
        }
    }

    Ok(value_to_primitive(value)?.into())
}

/// Convert a QuickJS value to a Stencila Primitive
fn value_to_primitive(value: Value) -> Result<Primitive, Error> {
    if value.is_undefined() || value.is_null() {
        Ok(Primitive::Null(Null))
    } else if let Some(value) = value.as_bool() {
        Ok(Primitive::Boolean(value))
    } else if let Some(value) = value.as_int() {
        Ok(Primitive::Integer(value as i64))
    } else if let Some(value) = value.as_big_int() {
        value
            .clone()
            .to_i64()
            .map(Primitive::Integer)
            .map_err(|_| Error::new_from_js(value.type_name(), "Integer"))
    } else if let Some(value) = value.as_float().or(value.as_number()) {
        Ok(Primitive::Number(value))
    } else if let Some(value) = value.as_string() {
        value
            .to_string()
            .map(Primitive::String)
            .map_err(|_| Error::new_from_js(value.type_name(), "String"))
    } else if let Some(value) = value.as_array() {
        let mut array = Vec::new();
        for item in value.iter().flatten() {
            let item = value_to_primitive(item)?;
            array.push(item);
        }
        Ok(Primitive::Array(Array(array)))
    } else if let Some(value) = value.as_object() {
        let mut object = IndexMap::new();
        for (name, value) in value.clone().into_iter().flatten() {
            let name = name.to_string().unwrap();
            let value = value_to_primitive(value)?;
            object.insert(name, value);
        }
        Ok(Primitive::Object(Object(object)))
    } else {
        Err(Error::new_from_js_message(
            value.type_name(),
            "Primitive",
            &"Unhandled JavaScript to Stencila Primitive conversion".to_string(),
        ))
    }
}

/// Convert a QuickJS value to a Stencila [`NodeType`] and [`Hint`]
fn value_to_type_hint(value: Value) -> (Option<String>, Option<Hint>) {
    let (node_type, hint) = if value.is_undefined() || value.is_null() {
        (NodeType::Null, None)
    } else if let Some(value) = value.as_bool() {
        (NodeType::Boolean, Some(Hint::Boolean(value)))
    } else if let Some(value) = value.as_int() {
        (NodeType::Integer, Some(Hint::Integer(value as i64)))
    } else if let Some(value) = value.as_big_int() {
        (
            NodeType::Integer,
            value.clone().to_i64().ok().map(Hint::Integer),
        )
    } else if let Some(value) = value.as_float().or(value.as_number()) {
        (NodeType::Number, Some(Hint::Number(value)))
    } else if let Some(value) = value.as_string() {
        (
            NodeType::String,
            value
                .to_string()
                .ok()
                .map(|value| Hint::StringHint(StringHint::new(value.chars().count() as i64))),
        )
    } else if let Some(value) = value.as_array() {
        (
            NodeType::Array,
            Some(Hint::ArrayHint(ArrayHint::new(value.len() as i64))),
        )
    } else if let Some(object) = value.as_object() {
        if let Ok(Some(node_type)) = object.get("type") {
            return (Some(node_type), None);
        }

        let mut keys = Vec::new();
        let mut hints = Vec::new();
        for key in object.keys::<String>().flatten() {
            if let Some(hint) = object
                .get(&key)
                .ok()
                .and_then(|value| value_to_type_hint(value).1)
            {
                keys.push(key);
                hints.push(hint);
            }
        }

        (
            NodeType::Object,
            Some(Hint::ObjectHint(ObjectHint::new(
                object.len() as i64,
                keys,
                hints,
            ))),
        )
    } else {
        return (None, None);
    };

    (Some(node_type.to_string()), hint)
}

/// Convert a Stencila Node to a QuickJS value
fn node_to_value<'js>(ctx: Ctx<'js>, node: &Node) -> Result<Value<'js>, Error> {
    Ok(match node {
        Node::Null(..) => Value::new_null(ctx),
        Node::Boolean(value) => Value::new_bool(ctx, *value),
        Node::Integer(value) => Value::from_big_int(BigInt::from_i64(ctx, *value)?),
        Node::UnsignedInteger(value) => Value::from_big_int(BigInt::from_u64(ctx, *value)?),
        Node::Number(value) => Value::new_number(ctx, *value),
        Node::String(value) => Value::from_string(JsString::from_str(ctx, value)?),
        Node::Array(value) => array_to_js_array(ctx, value)?,
        Node::Object(value) => object_to_js_object(ctx, value)?,
        _ => {
            let json = serde_json::to_string(node).map_err(|error| Error::IntoJs {
                from: "Node",
                to: "Value",
                message: Some(error.to_string()),
            })?;
            ctx.json_parse(json)?
        }
    })
}

/// Convert a Stencila Primitive to a QuickJS value
fn primitive_to_value<'js>(ctx: Ctx<'js>, primitive: &Primitive) -> Result<Value<'js>, Error> {
    Ok(match primitive {
        Primitive::Null(..) => Value::new_null(ctx),
        Primitive::Boolean(value) => Value::new_bool(ctx, *value),
        Primitive::Integer(value) => Value::from_big_int(BigInt::from_i64(ctx, *value)?),
        Primitive::UnsignedInteger(value) => Value::from_big_int(BigInt::from_u64(ctx, *value)?),
        Primitive::Number(value) => Value::new_number(ctx, *value),
        Primitive::String(value) => Value::from_string(JsString::from_str(ctx, value)?),
        Primitive::Array(value) => array_to_js_array(ctx, value)?,
        Primitive::Object(value) => object_to_js_object(ctx, value)?,
    })
}

/// Convert a Stencila Array to a QuickJS Array
fn array_to_js_array<'js>(ctx: Ctx<'js>, array: &Array) -> Result<Value<'js>, Error> {
    let js_array = JsArray::new(ctx.clone())?;
    for (index, item) in array.iter().enumerate() {
        js_array.set(index, primitive_to_value(ctx.clone(), item)?)?;
    }
    Ok(Value::from_array(js_array))
}

/// Convert a Stencila Object to a QuickJS Object
fn object_to_js_object<'js>(ctx: Ctx<'js>, object: &Object) -> Result<Value<'js>, Error> {
    let js_object = JsObject::new(ctx.clone())?;
    for (name, item) in object.iter() {
        js_object.set(name, primitive_to_value(ctx.clone(), item)?)?;
    }
    Ok(Value::from_object(js_object))
}

impl QuickJsKernelInstance {
    /// Create a new kernel instance
    fn new(id: String) -> Self {
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
            context: None,
            runtime: None,
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
                    self.name()
                );
                *previous = status;
                true
            } else {
                false
            }
        });

        Ok(())
    }

    /// Get the QuickJS runtime context for the kernel instance
    fn get_context(&mut self) -> Result<&mut AsyncContext> {
        self.context
            .as_mut()
            .ok_or_else(|| eyre!("Kernel not started yet"))
    }

    /// Run code in the kerne;
    async fn run_code(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Ok((vec![], vec![]));
        }

        self.set_status(KernelStatus::Busy)?;

        // Wrap code if begins and end with curly braces to avoid it being treated as a block
        let code = if code.starts_with('{') && code.ends_with('}') {
            ["(", code, ")"].concat()
        } else {
            code.to_string()
        };

        // Evaluate the code and convert any exception into a `ExecutionMessage`
        let (outputs, messages) = self
            .get_context()?
            .with(|ctx| {
                let console = Console::new();
                ctx.globals().set("console", console).ok();

                // Evaluate the code
                let result = ctx.eval::<Value, _>(code);

                let mut outputs: Vec<Node> = Vec::new();
                let mut messages: Vec<ExecutionMessage> = Vec::new();

                // Convert the outputs and messages captured using the `console` to
                // nodes and `ExecutionMessage`s. Doing this here ensure that any `console` calls
                // before any exceptions are in the right order
                let console: Console = ctx.globals().get("console").expect("was set above");
                for value in console.outputs {
                    match value_to_node(ctx.clone(), value) {
                        Ok(node) => outputs.push(node),
                        Err(error) => messages.push(ExecutionMessage::new(
                            MessageLevel::Error,
                            error.to_string(),
                        )),
                    }
                }
                for (level, message) in console.messages {
                    let level = match level {
                        0 => MessageLevel::Trace,
                        1 => MessageLevel::Debug,
                        2 => MessageLevel::Info,
                        3 => MessageLevel::Warning,
                        _ => MessageLevel::Error,
                    };
                    messages.push(ExecutionMessage::new(level, message));
                }

                // Evaluate the code and handle any outputs or exceptions
                match result {
                    Ok(value) => {
                        // Convert the
                        if !value.is_undefined() {
                            match value_to_node(ctx, value) {
                                Ok(node) => outputs.push(node),
                                Err(error) => messages.push(ExecutionMessage::new(
                                    MessageLevel::Error,
                                    error.to_string(),
                                )),
                            }
                        }
                    }
                    Err(..) => {
                        let exc = ctx.catch();
                        let message = if let Some(exc) = exc.as_exception() {
                            ExecutionMessage {
                                level: MessageLevel::Exception,
                                message: exc
                                    .message()
                                    .unwrap_or_else(|| "Unknown error".to_string()),
                                stack_trace: exc.stack(),
                                ..Default::default()
                            }
                        } else {
                            ExecutionMessage {
                                level: MessageLevel::Exception,
                                message: "Unknown error".to_string(),
                                ..Default::default()
                            }
                        };
                        messages.push(message)
                    }
                }

                (outputs, messages)
            })
            .await;

        self.set_status(KernelStatus::Ready)?;

        Ok((outputs, messages))
    }
}

/// A struct for a Javascript console
#[derive(Clone, Trace)]
#[rquickjs::class]
struct Console<'js> {
    /// The outputs captured from `console.log` calls
    outputs: Vec<Value<'js>>,

    /// The level (0=Trace,..4=Error) and message for messages
    messages: Vec<(u8, String)>,
}

#[rquickjs::methods]
impl<'js> Console<'js> {
    fn new() -> Console<'js> {
        Console {
            outputs: Vec::new(),
            messages: Vec::new(),
        }
    }

    fn log(&mut self, mut values: Rest<Value<'js>>) {
        self.outputs.append(&mut values)
    }

    #[qjs(rename = "trace")]
    fn trace_(&mut self, message: String) {
        self.messages.push((0, message.to_string()))
    }

    #[qjs()]
    fn debug(&mut self, message: String) {
        self.messages.push((1, message.to_string()))
    }

    #[qjs()]
    fn info(&mut self, message: String) {
        self.messages.push((2, message.to_string()))
    }

    #[qjs()]
    fn warn(&mut self, message: String) {
        self.messages.push((3, message.to_string()))
    }

    #[qjs()]
    fn error(&mut self, message: String) {
        self.messages.push((4, message.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{
        common::{indexmap::IndexMap, tokio},
        schema::{Array, ArrayHint, Hint, Node, Object, ObjectHint, Primitive, StringHint},
        tests::{create_instance, start_instance},
    };

    use super::*;

    // Pro-tip! Use get logs for these tests use:
    //
    // ```sh
    // RUST_LOG=trace cargo test -p kernel-JavaScript -- --nocapture
    // ```

    /// Standard kernel test for execution of code
    #[test_log::test(tokio::test)]
    async fn execution() -> Result<()> {
        let Some(instance) = create_instance::<QuickJsKernel>().await? else {
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
console.log(1);
console.log(2, 3);
2 + 2",
                    vec![
                        Node::Integer(1),
                        Node::Integer(2),
                        Node::Integer(3),
                        Node::Integer(4),
                    ],
                    vec![],
                ),
                // Variables set in one chunk are available in the next
                (
                    "
var a = 1;
var b = 2;
let c = 3;
const d = 4;",
                    vec![],
                    vec![],
                ),
                (
                    "
console.log(a, b, c, d)",
                    vec![
                        Node::Integer(1),
                        Node::Integer(2),
                        Node::Integer(3),
                        Node::Integer(4),
                    ],
                    vec![],
                ),
            ],
        )
        .await
    }

    /// Standard kernel test for evaluation of expressions
    #[test_log::test(tokio::test)]
    async fn evaluation() -> Result<()> {
        let Some(instance) = create_instance::<QuickJsKernel>().await? else {
            return Ok(());
        };

        kernel::tests::evaluation(
            instance,
            vec![
                ("1 + 1", Node::Integer(2), None),
                ("2.0 * 2.2", Node::Number(4.4), None),
                ("Math.sqrt(16)", Node::Integer(4), None),
                ("'a' + 'bc'", Node::String("abc".to_string()), None),
                ("'ABC'.toLowerCase()", Node::String("abc".to_string()), None),
                (
                    "[...[1, 2], 3]",
                    Node::Array(Array(vec![
                        Primitive::Integer(1),
                        Primitive::Integer(2),
                        Primitive::Integer(3),
                    ])),
                    None,
                ),
                (
                    "({...{a: 1}, ['b']: 2.3})",
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
                    Some("unexpected token in expression: '@'"),
                ),
                ("foo", Node::Null(Null), Some("'foo' is not defined")),
            ],
        )
        .await
    }

    /// Standard kernel test for printing nodes
    #[test_log::test(tokio::test)]
    async fn printing() -> Result<()> {
        let Some(instance) = create_instance::<QuickJsKernel>().await? else {
            return Ok(());
        };

        kernel::tests::printing(
            instance,
            r#"console.log('str')"#,
            r#"console.log('str1', 'str2')"#,
            r#"console.log(null, true, 1, 2.3, 'str', [1, 2.3, 'str'], {a:1, b:2.3, c:'str'})"#,
            r#"console.log({type:'Paragraph', content:[]})"#,
        )
        .await
    }

    /// Custom test for execution messages
    #[tokio::test]
    async fn messages() -> Result<()> {
        let Some(mut kernel) = start_instance::<QuickJsKernel>().await? else {
            return Ok(());
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("bad ^ # syntax").await?;
        assert_eq!(
            messages[0].message,
            "invalid first character of private name"
        );
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages[0].message, "'foo' is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

        // Console methods
        let (.., messages) = kernel
            .execute(
                r#"
console.debug("Debug message");
console.info("Info message");
console.warn("Warning message");
console.error("Error message");
"#,
            )
            .await?;

        assert_eq!(
            messages,
            vec![
                ExecutionMessage {
                    level: MessageLevel::Debug,
                    message: "Debug message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: MessageLevel::Info,
                    message: "Info message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: MessageLevel::Warning,
                    message: "Warning message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: MessageLevel::Error,
                    message: "Error message".to_string(),
                    ..Default::default()
                }
            ]
        );

        Ok(())
    }

    /// Standard kernel test for getting runtime information
    #[test_log::test(tokio::test)]
    async fn info() -> Result<()> {
        let Some(instance) = create_instance::<QuickJsKernel>().await? else {
            return Ok(());
        };

        let sw = kernel::tests::info(instance).await?;
        assert_eq!(sw.name, "QuickJS");
        assert!(sw.options.software_version.is_some());
        assert!(sw.options.operating_system.is_some());

        Ok(())
    }

    /// Standard kernel test for listing installed packages
    #[test_log::test(tokio::test)]
    async fn packages() -> Result<()> {
        let Some(instance) = start_instance::<QuickJsKernel>().await? else {
            return Ok(());
        };

        let pkgs = kernel::tests::packages(instance).await?;
        assert!(pkgs.is_empty());

        Ok(())
    }

    /// Standard kernel test for variable listing
    #[test_log::test(tokio::test)]
    async fn var_listing() -> Result<()> {
        let Some(instance) = create_instance::<QuickJsKernel>().await? else {
            return Ok(());
        };

        kernel::tests::var_listing(
            instance,
            r#"
var nul = null;
var bool = true;
var int = 123n;
var num = 1.23;
var str = "abc👍";
var arr = [1, 2, 3];
var obj = {a:1, b:2.3};
var para = {type: "Paragraph", content:[]}
"#,
            vec![
                Variable {
                    name: "nul".to_string(),
                    native_type: Some("null".to_string()),
                    node_type: Some("Null".to_string()),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "bool".to_string(),
                    native_type: Some("boolean".to_string()),
                    node_type: Some("Boolean".to_string()),
                    hint: Some(Hint::Boolean(true)),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "int".to_string(),
                    native_type: Some("bigint".to_string()),
                    node_type: Some("Integer".to_string()),
                    hint: Some(Hint::Integer(123)),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "num".to_string(),
                    native_type: Some("number".to_string()),
                    node_type: Some("Number".to_string()),
                    hint: Some(Hint::Number(1.23)),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "str".to_string(),
                    native_type: Some("string".to_string()),
                    node_type: Some("String".to_string()),
                    hint: Some(Hint::StringHint(StringHint::new(4))),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "arr".to_string(),
                    native_type: Some("array".to_string()),
                    node_type: Some("Array".to_string()),
                    hint: Some(Hint::ArrayHint(ArrayHint::new(3))),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "obj".to_string(),
                    native_type: Some("object".to_string()),
                    node_type: Some("Object".to_string()),
                    hint: Some(Hint::ObjectHint(ObjectHint::new(
                        2,
                        vec!["a".to_string(), "b".to_string()],
                        vec![Hint::Integer(1), Hint::Number(2.3)],
                    ))),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "para".to_string(),
                    native_type: Some("object".to_string()),
                    node_type: Some("Paragraph".to_string()),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
            ],
        )
        .await
    }

    /// Standard kernel test for variable management
    #[test_log::test(tokio::test)]
    async fn var_management() -> Result<()> {
        let Some(instance) = create_instance::<QuickJsKernel>().await? else {
            return Ok(());
        };

        kernel::tests::var_management(instance).await
    }

    /// Standard kernel test for forking
    #[test_log::test(tokio::test)]
    async fn forking() -> Result<()> {
        let Some(instance) = create_instance::<QuickJsKernel>().await? else {
            return Ok(());
        };

        kernel::tests::forking(instance).await
    }
}
