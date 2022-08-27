use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

use sqlx::{
    any::{AnyConnectOptions, AnyPool, AnyPoolOptions, AnyRow},
    Column, ConnectOptions, Row, TypeInfo,
};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        serde::Serialize,
        tracing::{self, log::LevelFilter},
    },
    stencila_schema::{
        ArrayValidator, BooleanValidator, CodeError, Datatable, DatatableColumn, IntegerValidator,
        Node, Null, Number, NumberValidator, StringValidator, ValidatorTypes,
    },
    Kernel, KernelSelector, KernelStatus, KernelTrait, KernelType, Task, TaskResult,
};

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
    connection_pool: Option<AnyPool>,

    /// Any select queries that have been assigned to variables
    #[serde(skip)]
    assigned_selects: HashMap<String, Datatable>,
}

impl SqlKernel {
    /// Create a new `SqlKernel`
    pub fn new(selector: &KernelSelector) -> Self {
        Self {
            config: selector.config.clone(),
            ..Default::default()
        }
    }

    /// Connect to the database
    ///
    /// This is called just-in-time (called by `exec`, `get` etc) as well as in `start`
    /// so that any errors during connection can be surfaced to the user as code chunk errors.
    async fn connect(&mut self) -> Result<()> {
        if let Some(_pool) = &self.connection_pool {
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
        if let Some(options) = options.as_mssql_mut() {
            options.log_statements(LevelFilter::Trace);
        }
        if let Some(options) = options.as_postgres_mut() {
            options.log_statements(LevelFilter::Trace);
        }
        if let Some(options) = options.as_sqlite_mut() {
            options.log_statements(LevelFilter::Trace);
        }

        let pool = AnyPoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await?;
        self.connection_pool = Some(pool);

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
        let _pool = self.connect().await?;
        let pool = self.connection_pool.as_ref().unwrap();
        let sql = format!("SELECT * FROM \"{}\"", name.replace('"', "-"));
        match sqlx::query(&sql).fetch_all(pool).await {
            Ok(rows) => Ok(any_rows_to_datatable(rows)),
            Err(error) => {
                bail!("No such symbol: {}", error)
            }
        }
    }

    async fn set(&mut self, _name: &str, _value: Node) -> Result<()> {
        todo!("Create a new table with name")
    }

    async fn exec_sync(&mut self, code: &str) -> Result<Task> {
        let mut task = Task::begin_sync();
        let mut outputs = Vec::new();
        let mut messages = Vec::new();

        match self.connect().await {
            Ok(..) => {
                let pool = self.connection_pool.as_ref().unwrap();
                match sqlx::query(code).fetch_all(pool).await {
                    Ok(rows) => {
                        let datatable = any_rows_to_datatable(rows);
                        outputs.push(datatable);
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

/// Transform a vector of `sqlx` [`AnyRow`]s to a Stencila [`Datatable`]
fn any_rows_to_datatable(rows: Vec<AnyRow>) -> Node {
    let rows_len = rows.len();
    let columns = if let Some(row) = rows.first() {
        row.columns()
            .iter()
            .map(|column| {
                let name = column.name().to_string();
                let type_name = column.type_info().name();
                let validator = match type_name {
                    "BOOLEAN" => {
                        Some(ValidatorTypes::BooleanValidator(BooleanValidator::default()))
                    }
                    "INTEGER" => {
                        Some(ValidatorTypes::IntegerValidator(IntegerValidator::default()))
                    }
                    "REAL" => Some(ValidatorTypes::NumberValidator(NumberValidator::default())),
                    "TEXT" => Some(ValidatorTypes::StringValidator(StringValidator::default())),
                    "BLOB" | "NULL" => None,
                    _ => {
                        tracing::debug!("Unhandled column type: {}", type_name);
                        None
                    }
                };
                (name, validator)
            })
            .collect()
    } else {
        Vec::new()
    };

    // Pre-allocate an vector of the size needed to hold all values and insert them in
    // column-first order
    let mut values: Vec<Node> = vec![Node::Null(Null {}); columns.len() * rows.len()];
    for (row_index, row) in rows.into_iter().enumerate() {
        for (col_index, (_name, validator)) in columns.iter().enumerate() {
            let position = col_index * rows_len + row_index;
            let value = match validator {
                Some(ValidatorTypes::BooleanValidator(..)) => {
                    row.try_get::<bool, usize>(col_index).map(Node::Boolean)
                }
                Some(ValidatorTypes::IntegerValidator(..)) => {
                    row.try_get::<i64, usize>(col_index).map(Node::Integer)
                }
                Some(ValidatorTypes::NumberValidator(..)) => row
                    .try_get::<f64, usize>(col_index)
                    .map(|num| Node::Number(Number(num))),
                Some(ValidatorTypes::StringValidator(..)) => {
                    row.try_get::<String, usize>(col_index).map(Node::String)
                }
                _ => row.try_get::<String, usize>(col_index).map(Node::String),
            };
            if let Ok(value) = value {
                values[position] = value;
            }
        }
    }

    let columns = columns
        .into_iter()
        .map(|(name, validator)| DatatableColumn {
            name,
            validator: validator.map(|validator| {
                Box::new(ArrayValidator {
                    items_validator: Some(Box::new(validator)),
                    ..Default::default()
                })
            }),
            values: values.drain(..rows_len).collect(),
            ..Default::default()
        })
        .collect();

    Node::Datatable(Datatable {
        columns,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{common::tokio, stencila_schema::Number, KernelTrait};
    use test_utils::{assert_json_eq, common::serde_json::json};

    #[tokio::test]
    async fn get_set_exec() -> Result<()> {
        let mut kernel = SqlKernel::new(&KernelSelector::default());

        match kernel.get("a").await {
            Ok(..) => bail!("Expected an error"),
            Err(error) => assert!(error.to_string().contains("does not exist")),
        };

        match kernel.set("a", Node::String("A".to_string())).await {
            Ok(..) => bail!("Expected an error"),
            Err(error) => assert!(error
                .to_string()
                .contains("Unable to convert string `A` to a number")),
        };

        kernel.set("a", Node::Number(Number(1.23))).await?;

        let a = kernel.get("a").await?;
        assert!(matches!(a, Node::Number(..)));
        assert_json_eq!(a, json!(1.23));

        let (outputs, errors) = kernel.exec("a * 2").await?;
        assert_json_eq!(outputs, json!([2.46]));
        assert_eq!(errors.len(), 0);

        let (outputs, errors) = kernel.exec("x * 2").await?;
        assert_eq!(outputs.len(), 0);
        assert_json_eq!(
            errors,
            json!([{"type": "CodeError", "errorMessage": "Undefined variable or function: x"}])
        );

        Ok(())
    }
}
