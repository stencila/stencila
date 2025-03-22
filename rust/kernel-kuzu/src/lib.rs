use std::path::{Path, PathBuf};

use kuzu::{Connection, Database, LogicalType, SystemConfig, Value};

use kernel::{
    Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
    common::{async_trait::async_trait, eyre::Result, tracing},
    format::Format,
    generate_id,
    schema::{
        ExecutionBounds, ExecutionMessage, Node, SoftwareApplication, SoftwareApplicationOptions,
        StringOrNumber,
    },
};
use kernel_jinja::JinjaKernelInstance;

mod from_kuzu;
pub use from_kuzu::*;

mod to_kuzu;
pub use to_kuzu::*;

pub use kuzu;

const NAME: &str = "kuzu";

/// A kernel for querying Kuzu databases.
#[derive(Default)]
pub struct KuzuKernel;

impl Kernel for KuzuKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Database
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Cypher]
    }

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        vec![
            ExecutionBounds::Main,
            // Fork is possible by creating a read-only connection
            // but Box is not possible because statements such as `COPY .. TO ..`
            // will write to the filesystem
            ExecutionBounds::Fork,
        ]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(KuzuKernelInstance::new()))
    }
}

#[derive(Default)]
pub struct KuzuKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jinja kernel instance used to render any Jinja templating
    jinja: JinjaKernelInstance,

    /// The path that the database is started in
    ///
    /// Used to prefix any relative db path passed in a `//db` comment
    /// in the execute method.
    directory: Option<PathBuf>,

    /// The database instance
    db: Option<Database>,
}

impl KuzuKernelInstance {
    /// Create a new instance
    pub fn new() -> Self {
        let id = generate_id(NAME);
        Self {
            // It is important to give the Jinja kernel the same id since
            // it acting as a proxy to this kernel and a different id can
            // cause deadlocks for variable requests
            jinja: JinjaKernelInstance::with_id(&id),

            id,
            directory: None,
            db: None,
        }
    }
}

#[async_trait]
impl KernelInstance for KuzuKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        self.directory = Some(directory.to_owned());

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing Kuzu statements");

        let db = match &self.db {
            Some(db) => db,
            None => {
                let mut db = ":memory:".to_string();
                for line in code.lines() {
                    if let Some(relative_path) = line
                        .strip_prefix("//db")
                        .or_else(|| line.strip_prefix("// db"))
                    {
                        let path = relative_path.trim();
                        db = match &self.directory {
                            Some(dir) => dir.join(path).to_string_lossy().to_string(),
                            None => path.to_string(),
                        }
                    }
                }

                let config = SystemConfig::default();
                let db = Database::new(db, config)?;

                self.db = Some(db);
                self.db.as_ref().expect("just set")
            }
        };

        let connection = Connection::new(&db)?;

        // Render any Jinja templating
        let code = if code.contains("{%") || code.contains("{{") && code.contains("}}") {
            let (rendered, messages) = self.jinja.execute(code).await?;
            if !messages.is_empty() {
                return Ok((Vec::new(), messages));
            }

            if let Some(Node::String(rendered)) = rendered.first() {
                rendered.to_string()
            } else {
                code.to_string()
            }
        } else {
            code.to_string()
        };

        // Return on the first error, otherwise treat the result of the last statement
        // as the output. Keep track of line numbers so that location or error can be
        // accurately reported.
        let mut output = None;
        let mut line_offset: usize = 0;
        for query in code.split(";") {
            if !query
                .lines()
                .map(|line| line.trim())
                .all(|line| line.starts_with("//") || line.is_empty())
            {
                match connection.query(query) {
                    Ok(result) => output = Some(result),
                    Err(error) => {
                        let leading_lines = query
                            .chars()
                            .take_while(|&c| c == '\n')
                            .count()
                            .saturating_sub(1);

                        return Ok((
                            Vec::new(),
                            vec![execution_message_from_error(
                                error,
                                line_offset + leading_lines,
                            )],
                        ));
                    }
                }
            }
            line_offset += if line_offset == 0 { 1 } else { 0 } + query.matches('\n').count();
        }

        let Some(output) = output else {
            return Ok(Default::default());
        };

        if output.get_num_columns() == 0 {
            return Ok(Default::default());
        }

        if output.get_column_data_types() == vec![LogicalType::String]
            && output.get_column_names() == vec!["result"]
            && output.get_num_tuples() == 1
        {
            let output = output
                .into_iter()
                .flatten()
                .next()
                .and_then(|value| match value {
                    Value::String(value) => Some(value),
                    _ => None,
                })
                .unwrap_or_default();
            return if output.starts_with("Table") && output.ends_with("has been created") {
                Ok(Default::default())
            } else {
                Ok((vec![Node::String(output)], Vec::new()))
            };
        }

        let output = if output
            .get_column_data_types()
            .iter()
            .all(|data_type| matches!(data_type, LogicalType::Node | LogicalType::Rel))
        {
            Node::ImageObject(cytoscape_from_query_result(output)?)
        } else {
            Node::Datatable(datatable_from_query_result(output)?)
        };

        Ok((vec![output], Vec::new()))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Kuzu runtime info");

        Ok(SoftwareApplication {
            name: "Kuzu".to_string(),
            version: Some(StringOrNumber::String(kuzu::VERSION.into())),
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
        self.jinja.variable_channel(requester, responder)
    }

    async fn replicate(&mut self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Self::new()))
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{
        common::tokio,
        schema::{CodeLocation, MessageLevel},
    };

    use super::*;

    #[tokio::test]
    async fn outputs() -> Result<()> {
        let mut kernel = KuzuKernelInstance::new();
        kernel.start_here().await?;

        let (outputs, messages) = kernel
            .execute(
                "
CREATE NODE TABLE Person(name STRING PRIMARY KEY, age INT64);
CREATE (:Person {name: 'Alice', age: 20});
CREATE (:Person {name: 'Bob', age: 30});
CREATE (:Person {name: 'Carol', age: 40});",
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![]);

        let (outputs, messages) = kernel.execute("MATCH (n:Person) RETURN n;").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs.len(), 1);
        assert!(matches!(outputs[0], Node::ImageObject(..)));

        let (outputs, messages) = kernel
            .execute("MATCH (n:Person) RETURN n.name, n.age;")
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs.len(), 1);
        assert!(matches!(outputs[0], Node::Datatable(..)));

        Ok(())
    }

    #[tokio::test]
    async fn errors() -> Result<()> {
        let mut kernel = KuzuKernelInstance::new();
        kernel.start_here().await?;

        let (outputs, messages) = kernel.execute("Bad syntax").await?;
        assert_eq!(
            messages,
            vec![ExecutionMessage {
                level: MessageLevel::Error,
                message: "Syntax error: Bad syntax".into(),
                code_location: Some(CodeLocation {
                    start_line: Some(0),
                    start_column: Some(0),
                    ..Default::default()
                }),
                ..Default::default()
            }]
        );
        assert!(outputs.is_empty());

        let (outputs, messages) = kernel
            .execute(
                "// Comment followed by empty line, statement, and syntax error

RETURN 1;

MATCH [);
",
            )
            .await?;
        assert_eq!(
            messages,
            vec![ExecutionMessage {
                level: MessageLevel::Error,
                message: "Syntax error: MATCH [)".into(),
                code_location: Some(CodeLocation {
                    start_line: Some(4),
                    start_column: Some(6),
                    ..Default::default()
                }),
                ..Default::default()
            }]
        );
        assert!(outputs.is_empty());

        let (outputs, messages) = kernel
            .execute(
                "// Comment followed by ok statement, empty line and runtime error
RETURN 1;

RETURN foo
",
            )
            .await?;
        assert_eq!(
            messages,
            vec![ExecutionMessage {
                level: MessageLevel::Error,
                message: "Variable foo is not in scope.".into(),
                ..Default::default()
            }]
        );
        assert!(outputs.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn comments() -> Result<()> {
        let mut kernel = KuzuKernelInstance::new();
        kernel.start_here().await?;

        let (.., messages) = kernel
            .execute(
                "
// Regression test for comment ending in semicolon;",
            )
            .await?;
        assert_eq!(messages, vec![]);

        Ok(())
    }
}
