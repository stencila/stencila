use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use kuzu::{Connection, Database, LogicalType, SystemConfig, Value};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, OptionExt, Result},
        once_cell::sync::Lazy,
        regex::Regex,
        tracing,
    },
    format::Format,
    generate_id,
    schema::{
        CodeLocation, ExecutionBounds, ExecutionMessage, MessageLevel, Node, Null,
        SoftwareApplication, SoftwareApplicationOptions, StringOrNumber,
    },
    Kernel, KernelInstance, KernelType, KernelVariableRequest, KernelVariableRequester,
    KernelVariableResponder,
};
use kernel_jinja::JinjaKernelInstance;

mod from_kuzu;
pub use from_kuzu::*;

mod to_kuzu;
pub use to_kuzu::*;

// Re-exports for convenience of dependant crates
pub use kernel;
pub use kuzu;

const NAME: &str = "kuzu";

/// A kernel for interacting with Kuzu graph databases.
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
            ExecutionBounds::Fork,
            ExecutionBounds::Box,
        ]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self, bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(match bounds {
            ExecutionBounds::Main => KuzuKernelInstance::main(false, None),
            ExecutionBounds::Fork => KuzuKernelInstance::fork(None),
            ExecutionBounds::Box => KuzuKernelInstance::r#box(None),
        }))
    }
}

#[derive(Debug)]
pub struct KuzuKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Jinja kernel instance used to render any Jinja templating
    jinja: JinjaKernelInstance,

    /// The path that the kernel is started in
    ///
    /// Used to capture the directory passed to `start()` to prefix any relative db path
    /// later passed in a `// @db` comment to `execute()``.
    directory: Option<PathBuf>,

    /// The path to the Kuzu database directory
    path: Option<PathBuf>,

    /// The execution bounds for the kernel instance
    ///
    /// Used to configure read-write/read-only when connecting to
    /// the database and to disallow certain Cypher statements (in case of `Box`)
    bounds: ExecutionBounds,

    /// Whether the database connection is read-only
    ///
    /// This is necessary in addition to the `bounds` property because it
    /// is possible to have a `Main` instance that is read-only and in fact this
    /// is required for replicating the database.
    read_only: bool,

    /// The default transform to apply to query results.
    ///
    /// Determines how Kuzu query results are converted to Stencila nodes.
    ///
    /// Can be overridden using a `// @out` comment e.g. `// @out graph`.
    /// If not defined, nor overridden, is determined by the types of
    /// columns in the result.
    transform: Option<QueryResultTransform>,

    /// The database instance
    database: Option<Arc<Database>>,

    /// The variables assigned in this kernel
    variables: HashMap<String, Node>,

    /// A channel for making variable requests to other kernels
    variable_channel: Option<(KernelVariableRequester, KernelVariableResponder)>,
}

impl KuzuKernelInstance {
    /// Create a new instance with `Main` execution bounds
    ///
    /// Can optionally be read-only, which is necessary to be able to
    /// replicate the kernel instance (i.e. to fork it)
    pub fn main(read_only: bool, path: Option<PathBuf>) -> Self {
        Self::init(ExecutionBounds::Main, read_only, path)
    }

    /// Create a new instance with `Fork` execution bounds
    pub fn fork(path: Option<PathBuf>) -> Self {
        Self::init(ExecutionBounds::Fork, true, path)
    }

    /// Create a new instance with `Box` execution bounds
    pub fn r#box(path: Option<PathBuf>) -> Self {
        Self::init(ExecutionBounds::Box, true, path)
    }

    /// Create a new instance with `Box` execution bounds and id
    pub fn box_with(id: String, transform: QueryResultTransform) -> Self {
        let mut instance = Self::init(ExecutionBounds::Box, true, None);
        instance.id = id;
        instance.transform = Some(transform);
        instance
    }

    /// Initialize a new instance
    fn init(bounds: ExecutionBounds, read_only: bool, path: Option<PathBuf>) -> Self {
        let id = generate_id(NAME);

        Self {
            // It is important to give the Jinja kernel the same id since
            // it acting as a proxy to this kernel and a different id can
            // cause deadlocks for variable requests
            jinja: JinjaKernelInstance::with_id(&id),

            id,
            bounds,
            read_only,
            transform: None,
            directory: None,
            path,
            database: None,
            variables: HashMap::new(),
            variable_channel: None,
        }
    }

    /// Set/reset the database path
    ///
    /// This does not create a database instance but does ensure that
    /// any existing one is dropped so that it will be created, to the correct
    /// path, when next needed.
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
        self.database = None;
    }

    /// Get, or instantiate, the Kuzu database instance
    pub fn database(&mut self) -> Result<Arc<Database>> {
        Ok(match &self.database {
            Some(database) => database.clone(),
            None => {
                let (path, read_only) = match &self.path {
                    Some(path) => (path.clone(), self.read_only),
                    // In-memory databases can not be read only
                    None => (PathBuf::from(":memory:"), false),
                };
                let config = SystemConfig::default().read_only(read_only);

                let database = Arc::new(Database::new(path, config)?);
                self.database = Some(database.clone());
                database
            }
        })
    }
}

#[async_trait]
impl KernelInstance for KuzuKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        tracing::trace!("Starting Kuzu kernel");

        self.directory = Some(directory.to_owned());

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing code in Kuzu kernel");

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

        // Request any parameters needed by the code
        static PARAM_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\$([a-zA-Z_][\w_]*)").expect("invalid regex"));
        let mut params: Vec<(String, Option<Value>)> = PARAM_REGEX
            .captures(&code)
            .iter()
            .map(|capture| (capture[1].to_string(), None))
            .collect();
        if !params.is_empty() {
            let (sender, receiver) = self
                .variable_channel
                .as_ref()
                .ok_or_eyre("Variable channel not yet setup")?;
            let mut receiver = receiver.resubscribe();

            // Get variable from this kernel, or make a request to get from elsewhere
            let mut requests_sent = false;
            for (name, value) in params.iter_mut() {
                if let Some(node) = self.variables.get(name) {
                    *value = Some(value_from_node(node)?);
                } else {
                    match sender.send(KernelVariableRequest {
                        variable: name.to_string(),
                        instance: self.id.clone(),
                    }) {
                        Err(error) => tracing::error!("While sending variable request: {error}"),
                        Ok(..) => {
                            tracing::trace!("Sent request for variable `{name}`");
                            requests_sent = true;
                        }
                    }
                }
            }

            if requests_sent {
                // Wait for responses
                tracing::trace!("Waiting for response for params");
                loop {
                    let response = receiver.recv().await?;

                    let mut all_some = true;
                    for (name, value) in params.iter_mut() {
                        if &response.variable == name && value.is_none() {
                            *value = Some(match &response.value {
                                Some(node) => value_from_node(node)?,
                                None => Value::Null(LogicalType::Any),
                            });
                        }

                        if value.is_none() {
                            all_some = false
                        }
                    }

                    if all_some {
                        break;
                    }
                }
                tracing::trace!("Got response for all params");
            }
        }

        // Search for a "// @db" line in the code specifying path, and optionally
        // read/write access, to database.
        static DB_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?m)^\/\/\s*@db\s+(?:(ro|rw)\s+)?(.+)$").expect("invalid regex")
        });
        if let Some(captures) = DB_REGEX.captures(&code) {
            if captures.get(1).map(|m| m.as_str()) == Some("ro") {
                self.read_only = true;
            }
            let relative_path = &captures[2];

            let path = match &self.directory {
                Some(dir) => dir.join(relative_path),
                None => relative_path.into(),
            };

            self.set_path(path);
        }

        let database = self.database()?;
        let connection = Connection::new(&database)?;

        // Ensure any necessary extensions are loaded. Any errors are intentionally ignored.
        if code.to_uppercase().contains("QUERY_FTS_INDEX") {
            connection.query("LOAD EXTENSION FTS;").ok();
        }

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
                // If `Box` execution bounds then do not allow statements which require filesystem access.
                if matches!(self.bounds, ExecutionBounds::Box)
                    && query
                        .lines()
                        .map(|line| line.trim())
                        .any(|line| line.starts_with("COPY"))
                {
                    return Ok((
                        Vec::new(),
                        vec![execution_message_for_copy(query, line_offset)],
                    ));
                }

                // Execute query and store result
                let result = if params.is_empty() {
                    connection.query(query)
                } else {
                    let mut statement = connection.prepare(query)?;
                    let params: Vec<(&str, Value)> = params
                        .iter()
                        .filter_map(|(name, value)| {
                            value.as_ref().map(|value| (name.as_str(), value.clone()))
                        })
                        .collect();
                    connection.execute(&mut statement, params)
                };
                let result = match result {
                    Ok(result) => result,
                    Err(error) => {
                        return Ok((
                            Vec::new(),
                            vec![execution_message_from_error(error, query, line_offset)],
                        ));
                    }
                };

                // If the query contains a `// @assign` comment then store the results
                // as a node. Because `QueryResult` is not clone-able it can
                // either be assigned or output, but not both
                static ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
                    Regex::new(r"(?m)^\/\/\s+*@assign\s+(\w+)(?:[ ]+(\w+))?")
                        .expect("invalid regex")
                });
                if let Some(captures) = ASSIGN_REGEX.captures(query) {
                    let name = &captures[1];
                    let transform = captures.get(2).map(|shape| shape.as_str()).unwrap_or("all");
                    let transform = QueryResultTransform::from_str(transform)?;
                    let node = node_from_query_result(result, Some(transform))?;
                    self.variables.insert(name.to_string(), node);
                } else {
                    output = Some(result)
                }
            }
            line_offset += if line_offset == 0 { 1 } else { 0 } + query.matches('\n').count();
        }

        // Early return if no output
        let Some(output) = output else {
            return Ok(Default::default());
        };

        // Early return if no columns in result
        if output.get_num_columns() == 0 {
            return Ok(Default::default());
        }

        // Early return for `CREATE` statements
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

        // Check if the default transform kind has been overridden
        static OUTPUT_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?m)^\/\/\s*@out\s+(\w+)$").expect("invalid regex"));
        let transform = if let Some(captures) = OUTPUT_REGEX.captures(&code) {
            Some(QueryResultTransform::from_str(&captures[1])?)
        } else {
            self.transform
        };

        let outputs = vec![node_from_query_result(output, transform)?];
        Ok((outputs, Vec::new()))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        // When evaluating an expression, force the transform to be a datatable
        // Do it this way to avoid adding to code.
        let transform = self.transform;
        self.transform = Some(QueryResultTransform::Datatable);

        let (nodes, messages) = self.execute(code).await?;

        // Reinstate default transform
        self.transform = transform;

        Ok((
            nodes
                .first()
                .map_or_else(|| Node::Null(Null), |node| node.clone()),
            messages,
        ))
    }

    async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        self.variables.insert(name.to_string(), value.clone());

        Ok(())
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        Ok(self.variables.get(name).cloned())
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Kuzu kernel info");

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
        self.variable_channel = Some((requester.clone(), responder.resubscribe()));

        self.jinja.variable_channel(requester, responder)
    }

    async fn replicate(&mut self, bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        tracing::trace!("Replicating Kuzu kernel");

        if self.path.is_none() {
            bail!("Can not replicate a Kuzu kernel for an in-memory database")
        }

        if !self.read_only {
            // It is not possible to have more than one database instance if one is read-write
            // See https://docs.kuzudb.com/concurrency/#limitations-of-creating-multiple-database-objects
            bail!("Can not replicate a Kuzu kernel with read-write access")
        }

        let path = self.path.clone();
        Ok(Box::new(match bounds {
            ExecutionBounds::Main => KuzuKernelInstance::main(true, path),
            ExecutionBounds::Fork => KuzuKernelInstance::fork(path),
            ExecutionBounds::Box => KuzuKernelInstance::r#box(path),
        }))
    }
}

fn execution_message_for_copy(query: &str, line_offset: usize) -> ExecutionMessage {
    let leading_lines = query
        .chars()
        .take_while(|&c| c == '\n')
        .count()
        .saturating_sub(1);

    ExecutionMessage {
        level: MessageLevel::Exception,
        message: "File system access not permitted with execution bounds `Box`".to_string(),
        code_location: Some(CodeLocation {
            start_line: Some((line_offset + leading_lines) as u64),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{
        common::{eyre::bail, tempfile::TempDir, tokio},
        schema::{Array, CodeLocation, MessageLevel, Primitive},
    };

    use super::*;

    #[tokio::test]
    async fn db_comment() -> Result<()> {
        let temp = TempDir::new()?;
        let path = temp.path().to_path_buf();

        {
            let mut kernel = KuzuKernelInstance::main(false, None);
            assert_eq!(kernel.path, None);
            assert_eq!(kernel.read_only, false);

            let (.., messages) = kernel
                .execute(&format!("// @db {}", path.display()))
                .await?;
            assert_eq!(messages, vec![]);
            assert_eq!(kernel.path, Some(path.clone()));
            assert_eq!(kernel.read_only, false);
        }

        {
            let mut kernel = KuzuKernelInstance::main(false, None);
            assert_eq!(kernel.path, None);
            assert_eq!(kernel.read_only, false);

            let (.., messages) = kernel
                .execute(&format!("\nRETURN 1;\n// @db ro {}", path.display()))
                .await?;
            assert_eq!(messages, vec![]);
            assert_eq!(kernel.path, Some(path));
            assert_eq!(kernel.read_only, true);
        }

        Ok(())
    }

    #[tokio::test]
    async fn assign_comment() -> Result<()> {
        let mut kernel = KuzuKernelInstance::main(false, None);
        kernel.execute("// @assign a val\nRETURN 1").await?;
        assert_eq!(kernel.get("a").await?, Some(Node::Integer(1)));

        kernel
            .execute("// previous\n// @assign b row\nRETURN 2.0")
            .await?;
        assert_eq!(
            kernel.get("b").await?,
            Some(Node::Array(Array(vec![Primitive::Number(2.0)])))
        );

        kernel
            .execute("// @assign c\nRETURN 3;\n\n//@assign d val\nRETURN 4")
            .await?;
        assert!(matches!(kernel.get("c").await?, Some(Node::Datatable(..))));
        assert_eq!(kernel.get("d").await?, Some(Node::Integer(4)));

        Ok(())
    }

    #[tokio::test]
    async fn outputs() -> Result<()> {
        let mut kernel = KuzuKernelInstance::main(false, None);

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
        let mut kernel = KuzuKernelInstance::main(false, None);

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
        let mut kernel = KuzuKernelInstance::main(false, None);

        let (.., messages) = kernel
            .execute(
                "
// Regression test for comment ending in semicolon;",
            )
            .await?;
        assert_eq!(messages, vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn bounds() -> Result<()> {
        let temp = TempDir::new()?;
        let path = temp.path().to_path_buf();

        // Main: can read and write and copy to
        let mut main = KuzuKernelInstance::main(false, Some(path.clone()));
        let (.., messages) = main
            .execute(&format!(
                "
CREATE NODE TABLE Person(name STRING PRIMARY KEY, age INT64);
CREATE (:Person {{name: 'Alice', age: 20}});
COPY (MATCH (p:Person) RETURN p) TO '{}';
",
                temp.path().join("a.csv").display()
            ))
            .await?;
        assert_eq!(messages, vec![]);
        drop(main); // Must drop the read-write connection before creating read-only connections in same process

        // Fork: can read and copy to, but not write
        let mut fork = KuzuKernelInstance::fork(Some(path.clone()));
        let (.., messages) = fork
            .execute(&format!(
                "
MATCH (p:Person) RETURN p;
COPY (MATCH (p:Person) RETURN p) TO '{}';
",
                temp.path().join("a.csv").display()
            ))
            .await?;
        assert_eq!(messages, vec![]);

        let (.., messages) = fork
            .execute("CREATE (:Person {name: 'Bob', age: 30});")
            .await?;
        assert_eq!(
            messages[0].message,
            "Connection exception: Cannot execute write operations in a read-only database!"
        );

        // Box: can read but not write or copy to or from
        let mut boxed = KuzuKernelInstance::r#box(Some(path));
        let (.., messages) = boxed.execute(r#"MATCH (p:Person) RETURN p;"#).await?;
        assert_eq!(messages, vec![]);

        let (.., messages) = boxed
            .execute(r#"CREATE (:Person {name: 'Bob', age: 30});"#)
            .await?;
        assert_eq!(
            messages[0].message,
            "Connection exception: Cannot execute write operations in a read-only database!"
        );

        let (.., messages) = boxed
            .execute(r#"COPY (MATCH (p:Person) RETURN p) TO 'some.csv';"#)
            .await?;
        assert_eq!(
            messages[0].message,
            "File system access not permitted with execution bounds `Box`"
        );

        Ok(())
    }

    #[tokio::test]
    async fn replicate() -> Result<()> {
        let temp = TempDir::new()?;
        let path = temp.path().to_path_buf();

        // Start a read-write instance that we can create a database in
        let mut main = KuzuKernelInstance::main(false, Some(path.clone()));
        let (.., messages) = main
            .execute(
                r#"
CREATE NODE TABLE Person(name STRING PRIMARY KEY, age INT64);
CREATE (:Person {name: 'Alice', age: 20});
"#,
            )
            .await?;
        assert_eq!(messages, vec![]);

        // Start a read-only instance that can be replicated
        let mut main = KuzuKernelInstance::main(true, Some(path.clone()));

        // Replicate with execution bounds `Fork`: can read and copy to, but not write

        let mut fork = main.replicate(ExecutionBounds::Fork).await?;
        let (.., messages) = fork
            .execute(&format!(
                r#"
MATCH (p:Person) RETURN p;
COPY (MATCH (p:Person) RETURN p) TO '{}';
"#,
                temp.path().join("a.csv").display()
            ))
            .await?;
        assert_eq!(messages, vec![]);

        let (.., messages) = fork
            .execute(r#"CREATE (:Person {name: 'Bob', age: 30});"#)
            .await?;
        assert_eq!(
            messages[0].message,
            "Connection exception: Cannot execute write operations in a read-only database!"
        );

        // Replicate with execution bounds `Box`: can read but not write or copy to or from

        let mut boxed = main.replicate(ExecutionBounds::Box).await?;
        let (.., messages) = boxed.execute("MATCH (p:Person) RETURN p;").await?;
        assert_eq!(messages, vec![]);

        let (.., messages) = boxed
            .execute("CREATE (:Person {name: 'Bob', age: 30});")
            .await?;
        assert_eq!(
            messages[0].message,
            "Connection exception: Cannot execute write operations in a read-only database!"
        );

        let (.., messages) = boxed
            .execute("COPY (MATCH (p:Person) RETURN p) TO 'some.csv';")
            .await?;
        assert_eq!(
            messages[0].message,
            "File system access not permitted with execution bounds `Box`"
        );

        Ok(())
    }

    #[tokio::test]
    async fn replicate_in_memory() -> Result<()> {
        let mut kernel = KuzuKernelInstance::main(true, None);

        for bounds in [
            ExecutionBounds::Main,
            ExecutionBounds::Fork,
            ExecutionBounds::Box,
        ] {
            match kernel.replicate(bounds).await {
                Ok(..) => bail!("expected error"),
                Err(error) => assert!(error
                    .to_string()
                    .starts_with("Can not replicate a Kuzu kernel for an in-memory database")),
            }
        }

        Ok(())
    }
}
