use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use kernel_docs::DocsKernelInstance;
use kernel_jinja::{
    error_to_execution_message,
    kernel::{
        common::{
            async_trait::async_trait,
            eyre::{bail, Result},
            itertools::Itertools,
            serde_json, tracing,
        },
        format::Format,
        generate_id,
        schema::{ExecutionBounds, ExecutionMessage, Node, SoftwareApplication},
        Kernel, KernelInstance, KernelType,
    },
    minijinja::{value::Object, Environment, UndefinedBehavior, Value},
};
use query::{add_to_env, Query};

mod query;

const NAME: &str = "docsql";

/// A kernel for querying document context databases
///
/// This kernel exposes a small domain specific language, "DocsQL", for querying
/// graph databases of Stencila Schema document nodes.
#[derive(Default)]
pub struct DocsQLKernel;

impl Kernel for DocsQLKernel {
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
    context: Arc<DocsQLKernelContext>,

    /// The path that the kernel is started in
    ///
    /// Used to determine the closest `.stencila` directory when
    /// instantiating the workspace context.
    directory: PathBuf,

    /// A [`DocsKernelInstance`] for the current workspace
    ///
    /// This is lazily instantiated because it can take a non-trivial
    /// amount of time.
    workspace: Option<DocsKernelInstance>,
}

impl ContextKernelInstance {
    fn new() -> Self {
        Self {
            id: generate_id(NAME),
            context: Arc::new(DocsQLKernelContext {}),
            directory: PathBuf::from("."),
            workspace: None,
        }
    }

    // Get the workspace kernel, instantiating it if necessary
    async fn workspace(&mut self) -> Result<&mut DocsKernelInstance> {
        if self.workspace.is_some() {
            Ok(self.workspace.as_mut().expect("checked above"))
        } else {
            let workspace = DocsKernelInstance::new_workspace(&self.directory).await?;
            self.workspace = Some(workspace);
            Ok(self.workspace.as_mut().expect("assigned above"))
        }
    }
}

#[async_trait]
impl KernelInstance for ContextKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        tracing::trace!("Starting context kernel");

        self.directory = directory.to_path_buf();

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing code in context kernel");

        let mut env = Environment::empty();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        add_to_env(&mut env);

        // Erase comment lines (but keep lines for line numbering)
        let code = code
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("#") {
                    ""
                } else {
                    line
                }
            })
            .join("\n");

        let expr = match env.compile_expression(&code) {
            Ok(expr) => expr,
            Err(error) => return Ok((Vec::new(), vec![error_to_execution_message(error)])),
        };

        let context = Value::from_dyn_object(self.context.clone());
        match expr.eval(context) {
            Ok(value) => {
                if let Some(query) = value.downcast_object::<Query>() {
                    match query.db.as_str() {
                        "workspace" => query.execute(self.workspace().await?).await,
                        _ => bail!("Unknown context database: {}", query.db),
                    }
                } else {
                    let value = serde_json::to_value(value)?;
                    let node: Node = serde_json::from_value(value)?;
                    Ok((vec![node], Vec::new()))
                }
            }
            Err(error) => Ok((Vec::new(), vec![error_to_execution_message(error)])),
        }
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

/// A Jinja context for the [`DocsQLKernel`]
#[derive(Debug)]
struct DocsQLKernelContext {}

impl Object for DocsQLKernelContext {}
