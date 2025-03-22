use std::path::Path;

use kuzu::{Connection, Database, LogicalType, SystemConfig, Value};

use kernel::{
    Kernel, KernelInstance, KernelType,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
        tracing,
    },
    format::Format,
    generate_id,
    schema::{
        ExecutionBounds, ExecutionMessage, Node, SoftwareApplication, SoftwareApplicationOptions,
        StringOrNumber,
    },
};

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

    fn create_instance(&self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(KuzuKernelInstance::new()))
    }
}

#[derive(Default)]
pub struct KuzuKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The path to the database
    path: Option<String>,

    /// The database instance
    db: Option<Database>,
}

impl KuzuKernelInstance {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            id: generate_id(NAME),
            path: None,
            db: None,
        }
    }
}

#[async_trait]
impl KernelInstance for KuzuKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, _directory: &Path) -> Result<()> {
        let config = SystemConfig::default();
        let db = Database::in_memory(config)?;

        self.db = Some(db);

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing Kuzu statements");

        let Some(db) = &self.db else {
            bail!("Database has not been started");
        };
        let connection = Connection::new(&db)?;

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
