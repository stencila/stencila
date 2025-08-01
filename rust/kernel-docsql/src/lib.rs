use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex as SyncMutex, MutexGuard as SyncMutexGuard},
};

use kernel_docsdb::{DocsDBChannels, DocsDBKernelInstance};
use kernel_jinja::{
    self, kernel::{
        common::{
            async_trait::async_trait, eyre::{eyre, OptionExt, Result}, itertools::Itertools, once_cell::sync::Lazy, regex::Regex, serde_json, tokio::sync::Mutex, tracing
        }, format::Format, generate_id, schema::{ExecutionBounds, ExecutionMessage, MessageLevel, Node, SoftwareApplication}, Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder
    }, minijinja::{context, Environment, Error, ErrorKind, UndefinedBehavior, Value}, JinjaKernelContext
};

mod cypher;
mod docsql;
mod github;
mod nodes;
mod openalex;
mod subquery;

use cypher::{
    CypherQuery, CypherQueryLabelled, CypherQueryNodeType, CypherQuerySectionType,
    CypherQueryVariables, add_document_functions,
};
use docsql::{add_constants, add_functions};
use github::{GitHubQuery, add_github_functions};
use nodes::{NodeProxies, NodeProxy};
use openalex::{OpenAlexQuery, add_openalex_functions};
use subquery::add_subquery_functions;

use crate::docsql::{GLOBAL_NAMES, strip_comments};

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
        vec![Format::DocsQL]
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
        Ok(Box::new(DocsQLKernelInstance::new(None, None)?))
    }
}

pub struct DocsQLKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jinja context used to request variables from other kernels
    context: Option<Arc<JinjaKernelContext>>,

    /// The path that the kernel is started in
    ///
    /// Used to determine the closest `.stencila` directory when
    /// instantiating the workspace context.
    directory: PathBuf,

    /// A [`DocsDBKernelInstance`] for the current document
    document: Option<Arc<Mutex<DocsDBKernelInstance>>>,

    /// A [`DocsDBKernelInstance`] for the current workspace
    ///
    /// This is lazily instantiated because it can take a non-trivial
    /// amount of time.
    workspace: Option<Arc<Mutex<DocsDBKernelInstance>>>,
}

impl DocsQLKernelInstance {
    pub fn new(directory: Option<PathBuf>, channels: Option<DocsDBChannels>) -> Result<Self> {
        let directory = directory.unwrap_or_else(|| PathBuf::from("."));

        let id = generate_id(NAME);

        let document = match channels {
            Some(channels) => Some(Arc::new(Mutex::new(DocsDBKernelInstance::new_document(
                &id, channels,
            )?))),
            None => None,
        };

        Ok(Self {
            id,
            context: None,
            directory,
            document,
            workspace: None,
        })
    }

    /// Get the workspace kernel, instantiating it if necessary
    async fn workspace(&mut self) -> Result<Arc<Mutex<DocsDBKernelInstance>>> {
        if let Some(workspace) = &self.workspace {
            Ok(workspace.clone())
        } else {
            let workspace = Arc::new(Mutex::new(
                DocsDBKernelInstance::new_workspace(&self.id, &self.directory).await?,
            ));
            self.workspace = Some(workspace.clone());
            Ok(workspace)
        }
    }
}

#[async_trait]
impl KernelInstance for DocsQLKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        tracing::trace!("Starting DocsQL kernel");

        self.directory = directory.to_path_buf();

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing code in DocsQL kernel");

        let mut env = Environment::empty();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        add_constants(&mut env);
        add_functions(&mut env);

        let messages = Arc::new(SyncMutex::new(Vec::new()));

        if let Some(document) = &self.document {
            let document = Arc::new(CypherQuery::new(
                "document".into(),
                document.clone(),
                messages.clone(),
            ));

            add_document_functions(&mut env, document.clone());

            env.add_global("document", Value::from_dyn_object(document));
        }

        if code.contains("workspace") {
            env.add_global(
                "workspace",
                Value::from_object(CypherQuery::new(
                    "workspace".into(),
                    self.workspace().await?,
                    messages.clone(),
                )),
            );
        }

        add_subquery_functions(&mut env);
        add_openalex_functions(&mut env, &messages);
        add_github_functions(&mut env, &messages);

        #[cfg(debug_assertions)]
        if code.contains("test") {
            env.add_global(
                "test",
                Value::from_object(CypherQuery::new(
                    "test".into(),
                    Arc::new(Mutex::new(DocsDBKernelInstance::new(None, None, None)?)),
                    messages.clone(),
                )),
            );
        }

        let code = strip_comments(code);
        if code.trim().is_empty() {
            return Ok(Default::default());
        }

        let should_use_db_method = |db: &str| {
            Ok((
                Vec::new(),
                vec![ExecutionMessage::new(
                    MessageLevel::Error,
                    format!(
                        "Document database should have a method called on it, e.g. `{db}.figures()`",
                    ),
                )],
            ))
        };

        let mut outputs = Vec::new();
        let mut line_offset = 0;
        for statement in code.split(";") {
            let lines = statement.lines().count();

            if statement.trim().is_empty() {
                line_offset += lines;
                continue;
            }

            static ASSIGN: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"^\s*let\s+([a-zA-Z][\w_]*)\s*=\s*(.+)").expect("invalid regex")
            });

            let (name, expr) = match ASSIGN.captures(statement) {
                Some(captures) => {
                    let name = captures[1].to_string();
                    let expr = statement
                        .replacen("let", "   ", 1)
                        .replacen(&name, &" ".repeat(name.len()), 1)
                        .replacen("=", " ", 1);
                    (Some(name), expr)
                }
                None => (None, statement.to_string()),
            };

            let expr = docsql::encode_filters(&expr);
            let expr = match env.compile_expression(&expr) {
                Ok(expr) => expr,
                Err(error) => {
                    return Ok((
                        Vec::new(),
                        vec![error_to_execution_message(error, line_offset)],
                    ));
                }
            };

            let context = match self.context.as_ref() {
                Some(context) => Value::from_dyn_object(context.clone()),
                None => context!(),
            };

            let value = match expr.eval(context) {
                Ok(value) => value,
                Err(error) => {
                    return Ok((
                        Vec::new(),
                        vec![error_to_execution_message(error, line_offset)],
                    ));
                }
            };

            line_offset += lines;

            // Attempt to convert from minijinja as simple primitive types before
            // attempting downcasts
            let mut nodes = if value.is_none() {
                Vec::new()
            } else if value.is_integer() {
                // Uses is_integer, rather than as_i64 first so that the minijinja ValueRepr::F64
                // does not get captured here (as_i64 uses try_from).
                let int = value.as_i64().ok_or_eyre("Unable to convert to integer")?;
                vec![Node::Integer(int)]
            } else if let Some(string) = value.as_str() {
                vec![Node::String(string.into())]
            } else if value.is_undefined() {
                let messages = messages
                    .lock()
                    .map_err(|error| eyre!(error.to_string()))?
                    .to_owned();
                let messages = if messages.is_empty() {
                    vec![ExecutionMessage::new(
                        MessageLevel::Error,
                        "Expression evaluates to undefined value".into(),
                    )]
                } else {
                    messages
                };
                return Ok((Vec::new(), messages));
            } else if value.downcast_object_ref::<CypherQueryLabelled>().is_some()
                || value
                    .downcast_object_ref::<CypherQueryVariables>()
                    .is_some()
                || value
                    .downcast_object_ref::<CypherQuerySectionType>()
                    .is_some()
                || value.downcast_object_ref::<CypherQueryNodeType>().is_some()
            {
                return Ok((
                    Vec::new(),
                    vec![ExecutionMessage::new(
                        MessageLevel::Error,
                        format!(
                            "Document query function should be called, use `{}()`",
                            statement.trim()
                        ),
                    )],
                ));
            } else if let Some(query) = value.downcast_object::<CypherQuery>() {
                if query.is_base() {
                    return should_use_db_method(statement.trim());
                }
                query.nodes()
            } else if let Some(query) = value.downcast_object::<OpenAlexQuery>() {
                if query.is_base() {
                    return should_use_db_method(statement.trim());
                }
                query.nodes()
            } else if let Some(query) = value.downcast_object::<GitHubQuery>() {
                if query.is_base() {
                    return should_use_db_method(statement.trim());
                }
                query.nodes()
            } else if let Some(proxies) = value.downcast_object::<NodeProxies>() {
                proxies.nodes()
            } else if let Some(proxy) = value.downcast_object::<NodeProxy>() {
                proxy.nodes()
            } else {
                // Coerce to a Stencila node via JSON as a last resort
                let value = serde_json::to_value(value)?;
                let node: Node = serde_json::from_value(value)?;
                vec![node]
            };

            if let (Some(name), Some(node)) = (name, nodes.first()) {
                self.set(&name, node).await?;
            } else {
                outputs.append(&mut nodes)
            }
        }

        // Resist the temptation to collect these messages before `query.nodes()` is called
        // above because that may add messages
        let messages = messages
            .lock()
            .map_err(|error| eyre!(error.to_string()))?
            .to_owned();

        Ok((outputs, messages))
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        if let Some(document) = &self.document {
            if let Some(node) = document.lock().await.get(name).await? {
                return Ok(Some(node));
            }
        }

        if let Some(workspace) = &self.workspace {
            if let Some(node) = workspace.lock().await.get(name).await? {
                return Ok(Some(node));
            }
        }

        if let Some(context) = &self.context {
            if let Some(node) = context.get_variable(name)? {
                return Ok(Some(node));
            }
        };

        Ok(None)
    }

    async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        if let Some(document) = &self.document {
            document.lock().await.set(name, value).await?;
        }

        if let Some(workspace) = &self.workspace {
            workspace.lock().await.set(name, value).await?;
        }

        if let Some(context) = &self.context {
            context.set_variable(name, value)?;
        }

        Ok(())
    }

    async fn remove(&mut self, name: &str) -> Result<()> {
        if let Some(document) = &self.document {
            document.lock().await.remove(name).await?;
        }

        if let Some(workspace) = &self.workspace {
            workspace.lock().await.remove(name).await?;
        }

        if let Some(context) = &self.context {
            context.remove_variable(name)?;
        }

        Ok(())
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
            GLOBAL_NAMES,
        )));
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting DocsQL kernel info");

        Ok(SoftwareApplication {
            name: "DocsQL Kernel".to_string(),
            ..Default::default()
        })
    }

    async fn replicate(&mut self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Self::new(None, None)?))
    }
}

/// Convert and error into an execution message with appropriate line and column offsets
fn error_to_execution_message(
    error: kernel_jinja::minijinja::Error,
    line_offset: usize,
) -> ExecutionMessage {
    let mut message = kernel_jinja::error_to_execution_message(error);

    if let Some(location) = message.code_location.as_mut() {
        if let Some(start_line) = location.start_line.as_mut() {
            *start_line += line_offset as u64;
        }
        if let Some(end_line) = location.end_line.as_mut() {
            *end_line += line_offset as u64;
        }
    }

    message
}

fn lock_messages(
    messages: &SyncMutex<Vec<ExecutionMessage>>,
) -> Option<SyncMutexGuard<'_, Vec<ExecutionMessage>>> {
    match messages.lock() {
        Ok(messages) => Some(messages),
        Err(..) => {
            tracing::error!("Unable to lock messages");
            None
        }
    }
}

fn try_messages(messages: &SyncMutex<Vec<ExecutionMessage>>) -> Result<(), Error> {
    let Some(messages) = lock_messages(messages) else {
        return Ok(());
    };

    if !messages.is_empty() {
        let detail = messages.iter().map(|msg| &msg.message).join(". ");
        Err(Error::new(ErrorKind::InvalidOperation, detail))
    } else {
        Ok(())
    }
}

/// Are we currently testing this crate
///
/// During tests, rather than make a request return some fixed entity ids
/// this is particularly useful for snapshot tests to avoid potentially
/// changing ids.
fn testing() -> bool {
    std::env::var("CARGO_PKG_NAME") == Ok("kernel-docsql".to_string())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_testing() {
        assert!(super::testing());
    }
}
