use std::{
    sync::{Arc, Mutex},
    thread,
};

use minijinja::{context, value::Object, Environment, Error, Value};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{Report, Result},
        serde_json, tokio, tracing,
    },
    format::Format,
    generate_id,
    schema::{
        CodeLocation, ExecutionBounds, ExecutionMessage, MessageLevel, Node, Null,
        SoftwareApplication, SoftwareApplicationOptions,
    },
    Kernel, KernelInstance, KernelType, KernelVariableRequest, KernelVariableRequester,
    KernelVariableResponder,
};

// Re-exports for dependants
pub use kernel;
pub use minijinja;

const NAME: &str = "jinja";

/// A kernel for rendering Jinja templates.
#[derive(Default)]
pub struct JinjaKernel;

impl Kernel for JinjaKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Templating
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Jinja]
    }

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        vec![
            ExecutionBounds::Main,
            // Fork & Box supported because no state mutation,
            // or filesystem or network access in this kernel
            ExecutionBounds::Fork,
            ExecutionBounds::Box,
        ]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(JinjaKernelInstance::new()))
    }
}

#[derive(Debug, Default)]
pub struct JinjaKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jinja template context
    ///
    /// Instantiated (with variable request channel) when `variable_requester_responder`
    /// is called.
    context: Option<Arc<JinjaKernelContext>>,
}

impl JinjaKernelInstance {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            id: generate_id(NAME),
            context: None,
        }
    }

    /// Create a new instance with a specific id
    ///
    /// This constructor is needed when we have a Jinja kernel instance
    /// that is a child of another kernel instance and we need the
    /// child and parent to have the same is (for variable resolution)
    pub fn with_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            context: None,
        }
    }
}

#[async_trait]
impl KernelInstance for JinjaKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing Jinja template");

        let env = Environment::new();

        let context = match self.context.as_ref() {
            Some(context) => Value::from_dyn_object(context.clone()),
            None => context!(),
        };

        Ok(match env.render_str(code, context) {
            Ok(rendered) => (vec![Node::String(rendered)], Vec::new()),
            Err(error) => (vec![], vec![error_to_execution_message(error)]),
        })
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        tracing::trace!("Evaluating Jinja expression");

        let env = Environment::new();
        let expr = match env.compile_expression(code) {
            Ok(expr) => expr,
            Err(error) => return Ok((Node::Null(Null), vec![error_to_execution_message(error)])),
        };

        let context = match self.context.as_ref() {
            Some(context) => Value::from_dyn_object(context.clone()),
            None => context!(),
        };

        Ok(match expr.eval(context) {
            Ok(value) => {
                let value = serde_json::to_value(value).unwrap_or_default();
                let node: Node = serde_json::from_value(value).unwrap_or_default();
                (node, Vec::new())
            }
            Err(error) => (Node::Null(Null), vec![error_to_execution_message(error)]),
        })
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Jinja runtime info");

        Ok(SoftwareApplication {
            name: "Jinja".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                operating_system: Some(std::env::consts::OS.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    fn variable_channel(
        &mut self,
        requester: KernelVariableRequester,
        responder: KernelVariableResponder,
    ) {
        self.context = Some(Arc::new(JinjaKernelContext {
            instance: self.id().to_string(),
            variable_channel: Mutex::new((requester, responder)),
        }));
    }

    async fn replicate(&mut self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Self::new()))
    }
}

/// Create an [`ExecutionMessage`] from a [`minijinja::Error`]
pub fn error_to_execution_message(error: Error) -> ExecutionMessage {
    let error_type = Some(error.kind().to_string());
    let message = error
        .detail()
        .map_or_else(|| error.to_string(), String::from);

    let code_location =
        if let (Some(source), Some(range)) = (error.template_source(), error.range()) {
            let mut line = 0;
            let mut col = 0;
            let mut start = None;
            let mut end = None;

            for (index, ch) in source.char_indices() {
                if index == range.start {
                    start = Some((line, col));
                }
                if index == range.end {
                    end = Some((line, col));
                    break;
                }

                if ch == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }
            if end.is_none() && range.end == source.len() {
                end = Some((line, col));
            }

            if start.is_some() {
                Some(CodeLocation {
                    start_line: start.map(|(line, ..)| line as u64),
                    start_column: start.map(|(.., col)| col as u64),
                    end_line: end.map(|(line, ..)| line as u64),
                    end_column: end.map(|(.., col)| col as u64),
                    ..Default::default()
                })
            } else {
                None
            }
        } else if let Some(line) = error.line() {
            Some(CodeLocation {
                start_line: Some(line as u64),
                ..Default::default()
            })
        } else {
            None
        };

    let mut error = &error as &dyn std::error::Error;
    let mut stack_trace = String::new();
    while let Some(source) = error.source() {
        stack_trace.push_str(&format!("\n{:#}", source));
        error = source;
    }
    let stack_trace = if stack_trace.is_empty() {
        None
    } else {
        Some(stack_trace)
    };

    ExecutionMessage {
        level: MessageLevel::Exception,
        error_type,
        message,
        code_location,
        stack_trace,
        ..Default::default()
    }
}

/// A Jinja template context used to make requests for variable to other kernels
#[derive(Debug)]
pub struct JinjaKernelContext {
    /// The id of the kernel instance
    ///
    /// Required to make requests for variables from other contexts
    instance: String,

    /// A channel for making variable requests
    ///
    /// Needs to be `Mutex` because is used in an immutable method
    variable_channel: Mutex<(KernelVariableRequester, KernelVariableResponder)>,
}

impl JinjaKernelContext {
    pub fn new(
        instance: String,
        requester: KernelVariableRequester,
        responder: KernelVariableResponder,
    ) -> Self {
        Self {
            instance,
            variable_channel: Mutex::new((requester, responder)),
        }
    }
}

impl Object for JinjaKernelContext {
    fn get_value(self: &Arc<Self>, name: &Value) -> Option<Value> {
        let Ok(mut guard) = self.variable_channel.lock() else {
            return None;
        };

        let (sender, receiver) = &mut *guard;
        let mut receiver = receiver.resubscribe();

        // Send the request
        match sender.send(KernelVariableRequest {
            variable: name.to_string(),
            instance: self.instance.clone(),
        }) {
            Err(error) => {
                tracing::error!("While sending variable request: {error}");
                return None;
            }
            Ok(..) => {
                tracing::trace!("Sent request for variable `{name}`");
            }
        }

        // This seems to be necessary to "tick over" the Tokio runtime
        // to process the request sent above
        tokio::spawn(async {});

        // Wait for the response. Uses `blocking_recv` to must be done in a thread.
        let name = name.to_string();
        let receiving = thread::spawn(move || {
            tracing::trace!("Waiting for response for variable `{name}`");
            loop {
                let response = receiver.blocking_recv()?;
                if response.variable == name {
                    tracing::trace!("Received response for variable `{name}`");
                    return Ok::<Option<Node>, Report>(response.value);
                }
            }
        });
        match receiving.join() {
            Err(..) => {
                tracing::error!("Error joining variable request receiving thread");
                None
            }
            Ok(Err(error)) => {
                tracing::error!("While receiving variable request: {error}");
                None
            }
            Ok(Ok(node)) => match node {
                Some(node) => Some(Value::from_serialize(&node)),
                // Return `None` here (rather than `Some(Value::UNDEFINED)` as we did
                // previously) so that if the variable is not found with name, that filters and
                // functions in the `Environment` will be search (rather than stopping here)
                None => None,
            },
        }
    }
}
