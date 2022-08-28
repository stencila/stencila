use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

use sqlx::{any::AnyConnectOptions, ConnectOptions, PgPool, SqlitePool};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        once_cell::sync::Lazy,
        regex::Regex,
        serde::Serialize,
        tracing::{self, log::LevelFilter},
    },
    stencila_schema::{CodeError, Datatable, Node},
    Kernel, KernelSelector, KernelStatus, KernelTrait, KernelType, TagMap, Task, TaskResult,
};

mod postgres;
mod sqlite;

#[cfg(test)]
mod tests;

#[derive(Debug)]
enum MetaPool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

/// A kernel that executes SQL
#[derive(Debug, Default, Serialize)]
#[serde(crate = "kernel::common::serde")]
pub struct SqlKernel {
    /// The kernel configuration containing the database URL
    config: Option<String>,

    /// The directory that the kernel is started in
    ///
    /// Used to be able to resolve the path to SQLite files with relative paths
    directory: Option<PathBuf>,

    /// The kernel's database connection pool
    #[serde(skip)]
    pool: Option<MetaPool>,

    /// Datatables resulting from SELECT queries that have been assigned to variables
    #[serde(skip)]
    assigned: HashMap<String, Datatable>,
}

impl SqlKernel {
    /// Create a new `SqlKernel`
    pub fn new(selector: &KernelSelector) -> Self {
        Self {
            config: selector.config.clone(),
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

        // If the URL is for a SQLite and the file path is relative then make it
        // absolute using the directory that the kernel was started in.
        if let (Some(spec), Some(directory)) = (url.strip_prefix("sqlite://"), &self.directory) {
            let path = PathBuf::from(spec);
            if spec != ":memory:" && path.is_relative() {
                url = ["sqlite://", &directory.join(path).to_string_lossy()].concat();
            }
        }

        tracing::trace!("Connecting to database: {}", url);

        let mut options = AnyConnectOptions::from_str(&url)?;
        let pool = if let Some(options) = options.as_postgres_mut() {
            options.log_statements(LevelFilter::Trace);
            MetaPool::Postgres(PgPool::connect_with(options.clone()).await?)
        } else if let Some(options) = options.as_sqlite_mut() {
            options.log_statements(LevelFilter::Trace);
            MetaPool::Sqlite(SqlitePool::connect_with(options.clone()).await?)
        } else {
            bail!(
                "Unhandled database type `{:?}` for url: {}",
                options.kind(),
                url
            )
        };
        self.pool = Some(pool);

        Ok(())
    }
}

#[async_trait]
impl KernelTrait for SqlKernel {
    async fn spec(&self) -> Kernel {
        Kernel::new("sql", KernelType::Builtin, &["sql"], true, false, false)
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(KernelStatus::Idle)
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

        // Attempt to get a table or view with the same name
        let query = format!("SELECT * FROM \"{}\"", name.replace('"', "-"));
        if let Ok(datatable) = match pool {
            MetaPool::Postgres(pool) => postgres::query_to_datatable(&query, pool).await,
            MetaPool::Sqlite(pool) => sqlite::query_to_datatable(&query, pool).await,
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

        let datatable = match value {
            Node::Datatable(node) => node,
            _ => {
                bail!(
                    "Only Datatables can be set as symbols in a SQL database; got a `{}`",
                    value.as_ref()
                )
            }
        };

        match pool {
            MetaPool::Postgres(pool) => postgres::table_from_datatable(name, datatable, pool).await,
            MetaPool::Sqlite(pool) => sqlite::table_from_datatable(name, datatable, pool).await,
        }
        .map_err(|error| eyre!("While setting symbol `{}` in SQL kernel: {}", name, error))
    }

    async fn exec_sync(&mut self, code: &str, tags: Option<&TagMap>) -> Result<Task> {
        let mut task = Task::begin_sync();
        let mut outputs = Vec::new();
        let mut messages = Vec::new();

        match self.connect().await {
            Ok(..) => {
                let pool = self
                    .pool
                    .as_ref()
                    .expect("connect() should ensure connection");

                let result = match pool {
                    MetaPool::Postgres(pool) => postgres::query_to_datatable(code, pool).await,
                    MetaPool::Sqlite(pool) => sqlite::query_to_datatable(code, pool).await,
                };
                match result {
                    Ok(datatable) => {
                        if let Some(assigns) = tags.and_then(|tags| tags.get_value("assigns")) {
                            static REGEX: Lazy<Regex> = Lazy::new(|| {
                                Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*$")
                                    .expect("Unable to create regex")
                            });
                            if REGEX.is_match(&assigns) {
                                self.assigned.insert(assigns, datatable);
                            } else {
                                messages.push(CodeError {
                                    error_message: format!("The `@assigns` tag is invalid. It should be a single identifier matching regular expression `{}`", REGEX.as_str()),
                                    ..Default::default()
                                });
                            }
                        } else {
                            outputs.push(Node::Datatable(datatable));
                        }
                    }
                    Err(error) => {
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
            }
            Err(error) => {
                messages.push(CodeError {
                    error_message: error.to_string(),
                    ..Default::default()
                });
            }
        }

        task.end(TaskResult::new(outputs, messages));

        Ok(task)
    }
}
