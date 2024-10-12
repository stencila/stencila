use std::{
    fmt,
    sync::{Arc, Mutex},
    thread,
};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{Report, Result},
        minijinja::{
            self, context,
            value::{Object, ObjectKind, StructObject},
            Environment, Value,
        },
        serde_json, tokio, tracing,
    },
    format::Format,
    generate_id,
    schema::{
        ExecutionMessage, MessageLevel, Node, Null, SoftwareApplication, SoftwareApplicationOptions,
    },
    Kernel, KernelForks, KernelInstance, KernelVariableRequest, KernelVariableRequester,
    KernelVariableResponder,
};

const NAME: &str = "jinja";

/// A kernel for rendering Jinja templates.
#[derive(Default)]
pub struct JinjaKernel;

impl Kernel for JinjaKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Jinja]
    }

    fn supports_forks(&self) -> kernel::KernelForks {
        KernelForks::Yes
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(JinjaKernelInstance::new()))
    }
}

#[derive(Default)]
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

    /// Generate a stack trace from a minijinja error
    fn stack_trace(error: minijinja::Error) -> Option<String> {
        let mut error = &error as &dyn std::error::Error;

        let mut stack_trace = String::new();
        while let Some(source) = error.source() {
            stack_trace.push_str(&format!("\n{:#}", source));
            error = source;
        }

        if stack_trace.is_empty() {
            None
        } else {
            Some(stack_trace)
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
            Some(context) => Value::from(context.clone()),
            None => context!(),
        };

        let (rendered, messages) = match env.render_str(code, context) {
            Ok(rendered) => (rendered, Vec::new()),
            Err(error) => (
                code.to_string(), // Note that if error, still returning original code string
                vec![ExecutionMessage {
                    level: MessageLevel::Exception,
                    message: error.to_string(),
                    stack_trace: Self::stack_trace(error),
                    ..Default::default()
                }],
            ),
        };

        Ok((vec![Node::String(rendered)], messages))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        tracing::trace!("Evaluating Jinja expression");

        let env = Environment::new();
        let expr = match env.compile_expression(code) {
            Ok(expr) => expr,
            Err(error) => {
                return Ok((
                    Node::Null(Null),
                    vec![ExecutionMessage {
                        level: MessageLevel::Exception,
                        message: error.to_string(),
                        stack_trace: Self::stack_trace(error),
                        ..Default::default()
                    }],
                ))
            }
        };

        let context = match self.context.as_ref() {
            Some(context) => Value::from(context.clone()),
            None => context!(),
        };

        let (value, messages) = match expr.eval(context) {
            Ok(value) => {
                let value = serde_json::to_value(value).unwrap();
                let node: Node = serde_json::from_value(value).unwrap();
                (node, Vec::new())
            }
            Err(error) => (
                Node::Null(Null),
                vec![ExecutionMessage {
                    level: MessageLevel::Exception,
                    message: error.to_string(),
                    stack_trace: Self::stack_trace(error),
                    ..Default::default()
                }],
            ),
        };

        Ok((value, messages))
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

    async fn fork(&mut self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Self::new()))
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

impl fmt::Display for JinjaKernelContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<JinjaKernelContext>")
    }
}

impl Object for JinjaKernelContext {
    fn kind(&self) -> ObjectKind {
        ObjectKind::Struct(self)
    }
}

impl StructObject for JinjaKernelContext {
    fn get_field(&self, name: &str) -> Option<Value> {
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
                None => Some(Value::UNDEFINED),
            },
        }
    }
}
