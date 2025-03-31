use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex as SyncMutex},
};

use kernel_docsdb::DocsDBKernelInstance;
use kernel_jinja::{
    error_to_execution_message,
    kernel::{
        common::{
            async_trait::async_trait,
            eyre::{eyre, Result},
            itertools::Itertools,
            serde_json,
            tokio::sync::Mutex,
            tracing,
        },
        format::Format,
        generate_id,
        schema::{ExecutionBounds, ExecutionMessage, MessageLevel, Node, SoftwareApplication},
        Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
    },
    minijinja::{context, Environment, UndefinedBehavior, Value},
    JinjaKernelContext,
};
use query::{NodeProxies, NodeProxy, Query};

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
        Ok(Box::new(DocsQLKernelInstance::new()))
    }
}

struct DocsQLKernelInstance {
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
    workspace: Option<Arc<Mutex<DocsDBKernelInstance>>>,
}

impl DocsQLKernelInstance {
    fn new() -> Self {
        Self {
            id: generate_id(NAME),
            context: None,
            directory: PathBuf::from("."),
            workspace: None,
        }
    }

    /// Get the workspace kernel, instantiating it if necessary
    async fn workspace(&mut self) -> Result<Arc<Mutex<DocsDBKernelInstance>>> {
        if let Some(workspace) = &self.workspace {
            Ok(workspace.clone())
        } else {
            let workspace = Arc::new(Mutex::new(
                DocsDBKernelInstance::new_workspace(&self.directory).await?,
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

        let messages = Arc::new(SyncMutex::new(Vec::new()));

        if code.contains("workspace") {
            env.add_global(
                "workspace",
                Value::from_object(Query::new(
                    "workspace".into(),
                    self.workspace().await?,
                    messages.clone(),
                )),
            );
        }

        #[cfg(test)]
        if code.contains("test") {
            env.add_global(
                "test",
                Value::from_object(Query::new(
                    "test".into(),
                    Arc::new(Mutex::new(DocsDBKernelInstance::new())),
                    messages.clone(),
                )),
            );
        }

        let code = strip_comments(code);
        if code.trim().is_empty() {
            return Ok(Default::default());
        }

        let code = query::transform_filters(&code);

        let expr = match env.compile_expression(&code) {
            Ok(expr) => expr,
            Err(error) => return Ok((Vec::new(), vec![error_to_execution_message(error)])),
        };

        let context = match self.context.as_ref() {
            Some(context) => Value::from_dyn_object(context.clone()),
            None => context!(),
        };

        let value = match expr.eval(context) {
            Ok(value) => value,
            Err(error) => {
                return Ok((Vec::new(), vec![error_to_execution_message(error)]));
            }
        };

        let outputs = if let Some(query) = value.downcast_object::<Query>() {
            query.nodes()
        } else if let Some(proxies) = value.downcast_object::<NodeProxies>() {
            proxies.nodes()
        } else if let Some(proxy) = value.downcast_object::<NodeProxy>() {
            proxy.nodes()
        } else if value.is_undefined() {
            let messages = messages
                .lock()
                .map_err(|error| eyre!(error.to_string()))?
                .to_owned();
            let messages = if messages.is_empty() {
                vec![ExecutionMessage::new(
                    MessageLevel::Error,
                    "Query evaluates to undefined value".into(),
                )]
            } else {
                messages
            };
            return Ok((Vec::new(), messages));
        } else {
            let value = serde_json::to_value(value)?;
            let node: Node = serde_json::from_value(value)?;
            vec![node]
        };

        // Resist to temptation to collect these messages before `query.nodes()` is called
        // above because that may add messages
        let messages = messages
            .lock()
            .map_err(|error| eyre!(error.to_string()))?
            .to_owned();

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
        tracing::trace!("Getting DocSQL kernel info");

        Ok(SoftwareApplication {
            name: "DocSQL Kernel".to_string(),
            ..Default::default()
        })
    }

    async fn replicate(&mut self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Self::new()))
    }
}

/// Strips comments after any `//`
///
/// Note that this will may result in blank lines which is
/// intentional for maintaining line numbers
fn strip_comments(code: &str) -> String {
    code.lines()
        .map(|line| {
            if let Some(pos) = line.find("//") {
                &line[..pos]
            } else {
                line
            }
        })
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    use kernel_jinja::kernel::{
        common::{
            eyre::{bail, Result},
            tokio,
        },
        schema::CodeChunk,
        KernelInstance,
    };

    use common_dev::pretty_assertions::assert_eq;

    #[test]
    fn strip_comments() {
        use super::strip_comments as s;

        assert_eq!(s(""), "");
        assert_eq!(s("// comment\nA"), "\nA");
        assert_eq!(s("A\n// comment\nB"), "A\n\nB");
        assert_eq!(s("A // comment\nB//comment"), "A \nB");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn generate_cypher() -> Result<()> {
        let mut kernel = DocsQLKernelInstance::new();

        macro_rules! expect {
            ($code:literal, $cypher:literal) => {
                let code = [$code, ".explain()"].concat();
                match kernel.execute(&code).await?.0.first() {
                    Some(Node::CodeChunk(CodeChunk { code, .. })) => {
                        let code = code.lines().skip(1).join("\n");
                        assert_eq!(code.as_str(), $cypher)
                    }
                    _ => bail!("Expected a code chunk"),
                }
            };
        }

        expect!(
            "test",
            "MATCH (node)
RETURN *
LIMIT 10"
        );

        expect!(
            "test.tables()",
            "MATCH (`table`:`Table`)
RETURN `table`
LIMIT 10"
        );

        expect!(
            "test.cells().skip(3).limit(2)",
            "MATCH (cell:TableCell)
RETURN cell
SKIP 3
LIMIT 2"
        );

        expect!(
            "test.cells(@position < 3)",
            r#"MATCH (cell:TableCell)
WHERE cell.position < 3
RETURN cell
LIMIT 10"#
        );

        expect!(
            "test.cells(@text =~ 'a')",
            r#"MATCH (cell:TableCell)
WHERE regexp_matches(cell.text, 'a')
RETURN cell
LIMIT 10"#
        );

        expect!(
            "test.cells(@text !~ 'a')",
            r#"MATCH (cell:TableCell)
WHERE NOT regexp_matches(cell.text, 'a')
RETURN cell
LIMIT 10"#
        );

        expect!(
            "test.cells(@text ^= 'a')",
            r#"MATCH (cell:TableCell)
WHERE starts_with(cell.text, 'a')
RETURN cell
LIMIT 10"#
        );

        expect!(
            "test.cells(@text $= 'a')",
            r#"MATCH (cell:TableCell)
WHERE ends_with(cell.text, 'a')
RETURN cell
LIMIT 10"#
        );

        expect!(
            "test.cells(@text in 'a')",
            r#"MATCH (cell:TableCell)
WHERE contains('a', cell.text)
RETURN cell
LIMIT 10"#
        );

        expect!(
            "test.paragraphs(search = 'keyword')",
            r#"CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword')
RETURN node
LIMIT 10"#
        );

        expect!(
            "test.paragraphs(searchAll = 'keyword1 keyword2')",
            r#"CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword1 keyword2', conjunctive := true)
RETURN node
LIMIT 10"#
        );

        expect!(
            "test.paragraphs(@text ^= 'Word', search = 'keyword')",
            r#"CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword')
WHERE starts_with(node.text, 'Word')
RETURN node
LIMIT 10"#
        );

        Ok(())
    }
}
