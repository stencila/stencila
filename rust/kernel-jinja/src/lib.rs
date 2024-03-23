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
        serde_json, tracing,
    },
    format::Format,
    schema::{
        ExecutionMessage, MessageLevel, Node, Null, SoftwareApplication, SoftwareApplicationOptions,
    },
    Kernel, KernelInstance, KernelVariableRequest, KernelVariableRequester,
    KernelVariableResponder,
};

/// A kernel for compiling styles, including Tailwind classes and Jinja templates, into CSS.
#[derive(Default)]
pub struct JinjaKernel {}

impl Kernel for JinjaKernel {
    fn name(&self) -> String {
        "jinja".to_string()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Jinja]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::<JinjaKernelInstance>::default())
    }
}

#[derive(Default)]
pub struct JinjaKernelInstance {
    context: Option<Arc<JinjaKernelContext>>,
}

/// Generate a stack trace from a minijinja error
fn stack_trace(error: minijinja::Error) -> Option<String> {
    let mut error = &error as &dyn std::error::Error;

    let mut stack_trace = String::new();
    while let Some(source) = error.source() {
        stack_trace.push_str(&format!("\n{:#}", source));
        error = source;
    }

    Some(stack_trace)
}

#[async_trait]
impl KernelInstance for JinjaKernelInstance {
    fn name(&self) -> String {
        "jinja".to_string()
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
                    stack_trace: stack_trace(error),
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
                        stack_trace: stack_trace(error),
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
                    stack_trace: stack_trace(error),
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

    fn variable_requester_responder(
        &mut self,
        requester: KernelVariableRequester,
        responder: KernelVariableResponder,
    ) {
        self.context = Some(Arc::new(JinjaKernelContext {
            kernel: self.name(),
            variable_channel: Mutex::new((requester, responder)),
        }));
    }
}

#[derive(Debug)]
pub struct JinjaKernelContext {
    /// The name of the kernel
    ///
    /// Required to make requests for variables from other contexts 
    kernel: String,

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
            instance: self.kernel.clone(),
        }) {
            Err(error) => {
                tracing::error!("While sending variable request: {error}");
                return None;
            }
            Ok(..) => {
                tracing::trace!("Sent request for variable `{name}`");
            }
        }

        // Wait for the response
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
            Ok(Ok(node)) => Some(Value::from_serializable(&node)),
        }
    }
}

#[cfg(test)]
mod tests {
    
    

    
}
