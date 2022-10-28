use std::{
    collections::{HashMap, HashSet},
    env,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use sqlx::{any::AnyConnectOptions, ConnectOptions, PgPool, SqlitePool};

use kernel::{
    common::{
        async_trait::async_trait,
        defaults::Defaults,
        eyre::{bail, eyre, Result},
        itertools::Itertools,
        once_cell::sync::Lazy,
        regex::Regex,
        serde::Serialize,
        serde_with::skip_serializing_none,
        tokio::sync::{mpsc, RwLock},
        tracing::{self, log::LevelFilter},
    },
    formats::Format,
    graph_triples::ResourceChange,
    stencila_schema::{
        BlockContent, CodeChunk, CodeError, Datatable, Form, InlineContent, Node, Paragraph,
    },
    Kernel, KernelSelector, KernelStatus, KernelTrait, KernelType, TagMap, Task, TaskResult,
};

mod common;
mod duck;
mod postgres;
mod sqlite;

use duck::DuckPond;

#[cfg(test)]
mod tests;

static SYMBOL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*$").expect("Unable to create regex"));

static BINDING_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$([a-zA-Z_][a-zA-Z_0-9]*)").expect("Unable to create regex"));

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
enum MetaPool {
    Duck(DuckPond),
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

type WatchedTables = Arc<RwLock<HashSet<String>>>;

/// A kernel that executes SQL
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, Serialize)]
#[serde(crate = "kernel::common::serde")]
pub struct SqlKernel {
    /// The kernel configuration containing the database URL
    config: Option<String>,

    /// The directory that the kernel is started in
    ///
    /// Used to be able to resolve the path to SQLite files with relative paths
    directory: Option<PathBuf>,

    /// The URL of the database (resolved just-in-time before connecting)
    url: Option<String>,

    /// The kernel's database connection pool
    #[serde(skip)]
    pool: Option<MetaPool>,

    /// Datatables resulting from SELECT queries that have been assigned to variables
    #[serde(skip)]
    assigned: HashMap<String, Datatable>,

    /// Nodes, other than Datatables, that are `set()` in this kernel and available
    /// for SQL statement parameter bindings
    #[serde(skip)]
    parameters: HashMap<String, Node>,

    /// Whether a notification listening task has been started for this kernel
    watching: bool,

    /// The tables that the kernel is listening for change notifications for
    #[def = "Arc::new(RwLock::new(HashSet::new()))"]
    #[serde(skip)]
    watches: WatchedTables,

    /// A sender to send [`ResourceChange`]s back to the owning document (if any)
    #[serde(skip)]
    resource_changes_sender: Option<mpsc::Sender<ResourceChange>>,
}

impl SqlKernel {
    /// Create a new `SqlKernel`
    pub fn new(
        selector: &KernelSelector,
        resource_changes_sender: Option<mpsc::Sender<ResourceChange>>,
    ) -> Self {
        Self {
            config: selector.config.clone(),
            resource_changes_sender,
            ..Default::default()
        }
    }

    /// Connect to the database (if not already connected)
    ///
    /// This is called just-in-time (called by `exec`, `get` etc) as well as in `start`
    /// so that any errors during connection can be surfaced to the user as code chunk errors.
    async fn connect(&mut self) -> Result<()> {
        if self.pool.is_some() {
            return Ok(());
        }

        // Resolve the database URL, falling back to env var and then to in-memory SQLite
        let mut url = self
            .config
            .clone()
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "sqlite://:memory:".to_string());

        // If the URL is for a SQLite or DuckDb database and the file path is relative then make it
        // absolute using the directory that the kernel was started in.
        if let (Some(spec), Some(directory)) = (url.strip_prefix("sqlite://"), &self.directory) {
            let path = PathBuf::from(spec);
            if spec != ":memory:" && path.is_relative() {
                url = ["sqlite://", &directory.join(path).to_string_lossy()].concat();
            }
        } else if let (Some(spec), Some(directory)) =
            (url.strip_prefix("duckdb://"), &self.directory)
        {
            let path = PathBuf::from(spec);
            if spec != ":memory:" && path.is_relative() {
                url = ["duckdb://", &directory.join(path).to_string_lossy()].concat();
            }
        }

        tracing::trace!("Connecting to database: {}", url);

        let pool = if let Some(path) = url.strip_prefix("duckdb://") {
            let pond = DuckPond::connect(path);
            MetaPool::Duck(pond)
        } else {
            let mut options = AnyConnectOptions::from_str(&url)?;
            if let Some(options) = options.as_postgres_mut() {
                options.log_statements(LevelFilter::Trace);
                let pool = PgPool::connect_with(options.clone()).await?;
                MetaPool::Postgres(pool)
            } else if let Some(options) = options.as_sqlite_mut() {
                options.log_statements(LevelFilter::Trace);
                let pool = SqlitePool::connect_with(options.clone()).await?;
                MetaPool::Sqlite(pool)
            } else {
                bail!(
                    "Unhandled database type `{:?}` for url: {}",
                    options.kind(),
                    url
                )
            }
        };
        self.pool = Some(pool);
        self.url = Some(url);

        Ok(())
    }

    /// Listen for notifications from the database (if not already listening)
    async fn watch(&mut self, tables: &[String]) -> Result<()> {
        self.connect().await?;
        let pool = self
            .pool
            .as_ref()
            .expect("connect() should ensure connection");
        let url = self.url.as_ref().expect("connect() should ensure URL");

        tracing::debug!(
            "Watching tables `{}` in database `{}`",
            tables.join(", "),
            url
        );

        let sender = match &self.resource_changes_sender {
            Some(sender) => sender.to_owned(),
            None => bail!("No resource sender provided to this SQL kernel"),
        };

        if !self.watching {
            let watches = self.watches.clone();
            match pool {
                MetaPool::Duck(pool) => {
                    duck::watch(url, pool, watches, sender).await?;
                }
                MetaPool::Postgres(pool) => {
                    postgres::watch(url, pool, watches, sender).await?;
                }
                MetaPool::Sqlite(pool) => {
                    sqlite::watch(url, pool, watches, sender).await?;
                }
            }
            self.watching = true;
        }

        let mut watches = self.watches.write().await;

        if let Some(first) = tables.first() {
            if first == "@all" {
                let schema = tables.get(1);
                let tables = match pool {
                    MetaPool::Duck(pool) => duck::watch_all(schema, pool).await?,
                    MetaPool::Postgres(pool) => postgres::watch_all(schema, pool).await?,
                    MetaPool::Sqlite(pool) => sqlite::watch_all(schema, pool).await?,
                };
                for table in tables {
                    watches.insert(table);
                }
                return Ok(());
            }
        }

        for table in tables {
            if !watches.contains(table) {
                match pool {
                    MetaPool::Duck(pool) => duck::watch_table(table, pool).await?,
                    MetaPool::Postgres(pool) => postgres::watch_table(table, pool).await?,
                    MetaPool::Sqlite(pool) => sqlite::watch_table(table, pool).await?,
                }
                watches.insert(table.to_owned());
            }
        }

        Ok(())
    }

    /// Ignore notifications from the database for one or more tables
    ///
    /// This does not drop any triggers in the database as other documents may still want
    /// to be listening to the table
    async fn unwatch(&mut self, tables: &[String]) {
        let mut watches = self.watches.write().await;
        for table in tables {
            if table == "@all" {
                watches.clear();
                break;
            } else {
                watches.remove(table);
            }
        }
    }

    /// Split some SQL code into separate statements
    ///
    /// Strips whole-line and multi-line comments but not comments that are after SQL on a line.
    /// This is a crude implementation (multiline comments must stat/end at the start/end of a line)
    /// but should hopefully suffice in most cases.
    fn split_statements(sql: &str) -> Vec<String> {
        let mut code = String::new();
        let mut in_multiline_comment = false;
        for line in sql.lines() {
            let line = line.trim();

            if line.starts_with("--") {
                continue;
            } else if line.starts_with("/*") {
                in_multiline_comment = true;
                continue;
            } else if in_multiline_comment && line.ends_with("*/") {
                in_multiline_comment = false;
                continue;
            }

            code.push_str(line);
            code.push('\n');
        }
        code.split(';')
            .filter_map(|statement| {
                if statement.trim().is_empty() {
                    None
                } else {
                    Some(String::from(statement))
                }
            })
            .collect_vec()
    }
}

#[async_trait]
impl KernelTrait for SqlKernel {
    async fn spec(&self) -> Kernel {
        Kernel::new(
            "sql",
            KernelType::Builtin,
            &[Format::SQL, Format::PrQL],
            true,
            false,
            false,
        )
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(KernelStatus::Ready)
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        self.directory = Some(directory.to_owned());

        // Log error but do not return error so that another attempt is made to
        // re-connect is made in `exec` etc
        if let Err(error) = self.connect().await {
            tracing::error!("While attempting to connect to database: {}", error)
        }

        Ok(())
    }

    async fn get(&mut self, name: &str) -> Result<Node> {
        self.connect().await?;
        let pool = self
            .pool
            .as_ref()
            .expect("connect() should ensure connection");

        let params = &self.parameters;

        // Attempt to get a table or view with the same name
        let query = format!("SELECT * FROM \"{}\"", name.replace('"', "-"));
        if let Ok(datatable) = match pool {
            MetaPool::Duck(pool) => duck::query_to_datatable(&query, params, pool).await,
            MetaPool::Postgres(pool) => postgres::query_to_datatable(&query, params, pool).await,
            MetaPool::Sqlite(pool) => sqlite::query_to_datatable(&query, params, pool).await,
        } {
            return Ok(Node::Datatable(datatable));
        }

        // Attempt to get as a previously assigned symbol
        match self.assigned.get(name) {
            Some(datatable) => Ok(Node::Datatable(datatable.clone())),
            None => bail!("Unable to find symbol `{}` in database (it is not a table, view, or assigned query result)", name)
        }
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        self.connect().await?;
        let pool = self
            .pool
            .as_ref()
            .expect("connect() should ensure connection");

        if let Node::Datatable(datatable) = value {
            match pool {
                MetaPool::Duck(pool) => duck::table_from_datatable(name, datatable, pool).await,
                MetaPool::Postgres(pool) => {
                    postgres::table_from_datatable(name, datatable, pool).await
                }
                MetaPool::Sqlite(pool) => sqlite::table_from_datatable(name, datatable, pool).await,
            }
            .map_err(|error| eyre!("While setting table `{}` in SQL kernel: {}", name, error))
        } else {
            self.parameters.insert(name.to_string(), value);
            Ok(())
        }
    }

    async fn derive(&mut self, what: &str, from: &str) -> Result<Vec<Node>> {
        let parts: Vec<String> = from.split('.').map(String::from).collect();
        let (schema, table, column) = if parts.len() < 3 {
            (None, parts.first(), parts.get(1))
        } else {
            (parts.first(), parts.get(1), parts.get(2))
        };

        self.connect().await?;
        let pool = self
            .pool
            .as_ref()
            .expect("connect() should ensure connection");
        let url = self.url.as_ref().expect("connect() should ensure URL");

        let what = what.to_lowercase();
        if what == "parameter" {
            let column =
                column.ok_or_else(|| eyre!("A column name is required in derive from path"))?;
            let table =
                table.ok_or_else(|| eyre!("A table name is required in derive from path"))?;
            let schema = schema.map(|string| string.as_str());
            let parameter = match pool {
                MetaPool::Duck(pool) => {
                    duck::column_to_parameter(url, pool, column, table, schema).await?
                }
                MetaPool::Postgres(pool) => {
                    postgres::column_to_parameter(url, pool, column, table, schema).await?
                }
                MetaPool::Sqlite(pool) => {
                    sqlite::column_to_parameter(url, pool, column, table, schema).await?
                }
            };
            Ok(vec![Node::Parameter(parameter)])
        } else if what == "parameters" {
            let table =
                table.ok_or_else(|| eyre!("A table name is required in derive from path"))?;
            let schema = schema.map(|string| string.as_str());
            let parameters = match pool {
                MetaPool::Duck(pool) => duck::table_to_parameters(url, pool, table, schema).await?,
                MetaPool::Postgres(pool) => {
                    postgres::table_to_parameters(url, pool, table, schema).await?
                }
                MetaPool::Sqlite(pool) => {
                    sqlite::table_to_parameters(url, pool, table, schema).await?
                }
            };
            Ok(parameters.into_iter().map(Node::Parameter).collect())
        } else if what.starts_with("form") {
            let parts: Vec<_> = what.splitn(2, ':').collect();
            let action = parts.get(1);

            let table =
                table.ok_or_else(|| eyre!("A table name is required in derive from path"))?;
            let schema = schema.map(|string| string.as_str());

            let parameters = match pool {
                MetaPool::Duck(pool) => duck::table_to_parameters(url, pool, table, schema).await?,
                MetaPool::Postgres(pool) => {
                    postgres::table_to_parameters(url, pool, table, schema).await?
                }
                MetaPool::Sqlite(pool) => {
                    sqlite::table_to_parameters(url, pool, table, schema).await?
                }
            };

            let columns = parameters
                .iter()
                .map(|param| param.name.to_string())
                .collect_vec();
            let params = parameters
                .iter()
                .map(|param| ["$", &param.name].concat())
                .collect_vec();
            let mut content: Vec<BlockContent> = parameters
                .into_iter()
                .map(|param| {
                    BlockContent::Paragraph(Paragraph {
                        content: vec![InlineContent::Parameter(param)],
                        ..Default::default()
                    })
                })
                .collect();
            if let Some(action) = action {
                let sql = match *action {
                    "create" => {
                        format!("INSERT INTO \"{table}\"\nVALUES ({});", params.join(", "))
                    }
                    "update" => {
                        let sets = columns
                            .iter()
                            .map(|column| [column, " = $", column].concat())
                            .join("\n");
                        format!("UPDATE \"{table}\" SET {} WHERE ...;", sets)
                    }
                    "delete" => format!("DELETE FROM \"{table}\"\nWHERE ...;"),
                    _ => format!("-- Unknown form action '{action}'"),
                };
                content.push(BlockContent::CodeChunk(CodeChunk {
                    programming_language: "sql".to_string(),
                    text: sql,
                    ..Default::default()
                }))
            }

            let form = Form {
                content,
                ..Default::default()
            };

            Ok(vec![Node::Form(form)])
        } else {
            bail!("Do not know how to derive `{}` from database", what)
        }
    }

    async fn exec_sync(&mut self, code: &str, lang: Format, tags: Option<&TagMap>) -> Result<Task> {
        let sql = match lang {
            Format::SQL => code.to_string(),
            Format::PrQL => match prql_compiler::compile(code) {
                Ok(sql) => sql,
                Err(error) => {
                    let (message, ..) = prql_compiler::format_error(error, "<code>", code, false);

                    let mut task = Task::begin_sync();
                    task.end(TaskResult::syntax_error(&message));
                    return Ok(task);
                }
            },
            _ => bail!("Unexpected language for `SqlKernel`: {}", lang),
        };

        let mut task = Task::begin_sync();
        let mut outputs = Vec::new();
        let mut messages = Vec::new();

        if let Err(error) = self.connect().await {
            messages.push(CodeError {
                error_message: error.to_string(),
                ..Default::default()
            });
        } else {
            if let Some(tags) = tags {
                let unwatch = tags.get_items("unwatch");
                if !unwatch.is_empty() {
                    self.unwatch(&unwatch).await;
                }
            }

            let pool = self
                .pool
                .as_ref()
                .expect("connect() should ensure connection");

            let params = &self.parameters;

            let statements = Self::split_statements(&sql);
            for statement in statements {
                let error_maybe = if statement.trim_start().to_lowercase().starts_with("select") {
                    let result = match pool {
                        MetaPool::Duck(pool) => {
                            duck::query_to_datatable(&statement, params, pool).await
                        }
                        MetaPool::Postgres(pool) => {
                            postgres::query_to_datatable(&statement, params, pool).await
                        }
                        MetaPool::Sqlite(pool) => {
                            sqlite::query_to_datatable(&statement, params, pool).await
                        }
                    };
                    match result {
                        Ok(datatable) => {
                            if let Some(assigns) = tags.and_then(|tags| tags.get_value("assigns")) {
                                if SYMBOL_REGEX.is_match(&assigns) {
                                    self.assigned.insert(assigns, datatable);
                                } else {
                                    messages.push(CodeError {
                                            error_message: format!("The `@assigns` tag is invalid. It should be a single identifier matching regular expression `{}`", SYMBOL_REGEX.as_str()),
                                            ..Default::default()
                                        });
                                }
                            } else {
                                outputs.push(Node::Datatable(datatable));
                            }
                            None
                        }
                        Err(error) => Some(error),
                    }
                } else {
                    match pool {
                        MetaPool::Duck(pool) => {
                            duck::execute_statement(&statement, params, pool).await
                        }
                        MetaPool::Postgres(pool) => {
                            postgres::execute_statement(&statement, params, pool).await
                        }
                        MetaPool::Sqlite(pool) => {
                            sqlite::execute_statement(&statement, params, pool).await
                        }
                    }
                    .err()
                };

                if let Some(error) = error_maybe {
                    let message = error.to_string();
                    let message = message
                        .strip_prefix("error returned from database:")
                        .unwrap_or(&message);
                    let message = message.trim().to_string();

                    messages.push(CodeError {
                        error_message: message,
                        ..Default::default()
                    });
                }
            }

            if let Some(tags) = tags {
                // Apply any @watch tags _after_ statements in case the user wants the watch
                // to apply to a table that they just created.
                let watch = tags.get_items("watch");
                if !watch.is_empty() {
                    if let Err(error) = self.watch(&watch).await {
                        messages.push(CodeError {
                            error_message: format!(
                                "While setting up watch for tables `{}`: {}",
                                watch.join(" "),
                                error
                            ),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        task.end(TaskResult::new(outputs, messages));

        Ok(task)
    }
}
