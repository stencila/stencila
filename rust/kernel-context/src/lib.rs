use std::sync::Arc;

use kernel_jinja::{
    error_to_execution_message,
    kernel::{
        common::{async_trait::async_trait, eyre::Result, serde_json, tracing},
        format::Format,
        generate_id,
        schema::{ExecutionBounds, ExecutionMessage, Node, SoftwareApplication},
        Kernel, KernelInstance, KernelType,
    },
    minijinja::{value::Object, Environment, Value},
};
use query::{add_to_env, Query};

mod query;

const NAME: &str = "context";

/// A kernel for accessing context
///
/// This kernel exposes a small domain specific language (DSL) for querying
/// graph context databases. If a query returns context then the query builder
/// prefixes that context with the Cypher query as thus serves as few shot
/// examples for LLMs to perform their own queries on the context database.
#[derive(Default)]
pub struct ContextKernel;

impl Kernel for ContextKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Database
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![]
    }

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        vec![
            ExecutionBounds::Main,
            // Fork & Box supported because no state mutation,
            // or filesystem writes or network access in this kernel
            ExecutionBounds::Fork,
            ExecutionBounds::Box,
        ]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(ContextKernelInstance::new()))
    }
}

struct ContextKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jinja context
    context: Arc<ContextKernelContext>,
}

impl ContextKernelInstance {
    fn new() -> Self {
        Self {
            id: generate_id(NAME),
            context: Arc::new(ContextKernelContext {}),
        }
    }
}

#[async_trait]
impl KernelInstance for ContextKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing code in context kernel");

        let mut env = Environment::new();
        add_to_env(&mut env);

        let expr = match env.compile_expression(code) {
            Ok(expr) => expr,
            Err(error) => return Ok((Vec::new(), vec![error_to_execution_message(error)])),
        };

        let context = Value::from_dyn_object(self.context.clone());
        let nodes = match expr.eval(context) {
            Ok(value) => {
                if let Some(query) = value.downcast_object::<Query>() {
                    query.execute()
                } else {
                    let value = serde_json::to_value(value).unwrap_or_default();
                    let node: Node = serde_json::from_value(value).unwrap_or_default();
                    vec![node]
                }
            }
            Err(error) => return Ok((Vec::new(), vec![error_to_execution_message(error)])),
        };

        Ok((nodes, Vec::new()))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting context kernel info");

        Ok(SoftwareApplication {
            name: "Context Kernel".to_string(),
            ..Default::default()
        })
    }

    async fn replicate(&mut self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Self::new()))
    }
}

/// A Jinja context for the [`ContextKernel`]
#[derive(Debug)]
pub struct ContextKernelContext {}

impl Object for ContextKernelContext {}
