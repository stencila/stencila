use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use kernel_docsdb::DocsDBKernelInstance;
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
        schema::{
            shortcuts::t, CodeBlock, ExecutionBounds, ExecutionMessage, Node, Paragraph,
            SoftwareApplication,
        },
        Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
    },
    minijinja::{context, Environment, UndefinedBehavior, Value},
    JinjaKernelContext,
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

    /// The Jinja context used to request variables from other kernels
    context: Option<Arc<JinjaKernelContext>>,

    /// The path that the kernel is started in
    ///
    /// Used to determine the closest `.stencila` directory when
    /// instantiating the workspace context.
    directory: PathBuf,

    /// A [`DocsDBKernelInstance`] for the current workspace
    ///
    /// This is lazily instantiated because it can take a non-trivial
    /// amount of time.
    workspace: Option<DocsDBKernelInstance>,
}

impl ContextKernelInstance {
    fn new() -> Self {
        Self {
            id: generate_id(NAME),
            context: None,
            directory: PathBuf::from("."),
            workspace: None,
        }
    }

    // Get the workspace kernel, instantiating it if necessary
    async fn workspace(&mut self) -> Result<&mut DocsDBKernelInstance> {
        if self.workspace.is_some() {
            Ok(self.workspace.as_mut().expect("checked above"))
        } else {
            let workspace = DocsDBKernelInstance::new_workspace(&self.directory).await?;
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

        // Erase comment lines (but keep lines for line numbering), checking for @explain
        let mut explain = false;
        let code = code
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("#") {
                    if line.contains("@explain") {
                        explain = true;
                    }
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

        let context = match self.context.as_ref() {
            Some(context) => Value::from_dyn_object(context.clone()),
            None => context!(),
        };

        let mut explanation = None;
        let (mut outputs, messages) = match expr.eval(context) {
            Ok(value) => {
                if let Some(query) = value.downcast_object::<Query>() {
                    let cypher = query.generate();
                    if explain {
                        explanation = Some(vec![
                            Node::Paragraph(Paragraph::new(vec![t(
                                format!("This DocsQL is equivalent to executing the following Cypher in the {} graph database", query.db),
                            )])),
                            Node::CodeBlock(CodeBlock {
                                programming_language: Some("docs".into()),
                                code: format!("// @{}\n{}", query.db, cypher).into(),
                                ..Default::default()
                            }),
                        ]);
                    }
                    match query.db.as_str() {
                        "workspace" => {
                            let kernel = self.workspace().await?;
                            kernel.execute(&cypher).await?
                        }
                        _ => bail!("Unknown context database: {}", query.db),
                    }
                } else {
                    let value = serde_json::to_value(value)?;
                    let node: Node = serde_json::from_value(value)?;
                    (vec![node], Vec::new())
                }
            }
            Err(error) => return Ok((Vec::new(), vec![error_to_execution_message(error)])),
        };

        let outputs = if let Some(mut explanation) = explanation {
            explanation.append(&mut outputs);
            explanation
        } else {
            outputs
        };

        Ok((outputs, messages))
    }

    fn variable_channel(
        &mut self,
        requester: KernelVariableRequester,
        responder: KernelVariableResponder,
    ) {
        self.context = Some(Arc::new(JinjaKernelContext::new(
            self.id().to_string(),
            requester,
            responder,
        )));
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
