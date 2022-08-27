use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

use sqlx::{
    any::{AnyConnectOptions, AnyKind, AnyPool, AnyPoolOptions, AnyRow},
    Column, ConnectOptions, Row, TypeInfo,
};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
        serde::Serialize,
        serde_json,
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
    pool: Option<AnyPool>,

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

        let sql = format!("SELECT * FROM \"{}\"", name.replace('"', "-"));
        match sqlx::query(&sql).fetch_all(pool).await {
            Ok(rows) => Ok(any_rows_to_datatable(rows)),
            Err(error) => {
                bail!("While getting symbol `{}` from SQL kernel: {}", name, error)
            }
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

        let result = match pool.any_kind() {
            AnyKind::Postgres => datatable_to_postgres(name, datatable, pool).await,
            _ => datatable_to_other(name, datatable, pool).await,
        };

        match result {
            Ok(..) => Ok(()),
            Err(error) => {
                bail!("While setting symbol `{}` in SQL kernel: {}", name, error)
            }
        }
    }

    async fn exec_sync(&mut self, code: &str) -> Result<Task> {
        let mut task = Task::begin_sync();
        let mut outputs = Vec::new();
        let mut messages = Vec::new();

        match self.connect().await {
            Ok(..) => {
                let pool = self
                    .pool
                    .as_ref()
                    .expect("connect() should ensure connection");
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
                Some(ValidatorTypes::BooleanValidator(..)) => row
                    .try_get::<bool, usize>(col_index)
                    .map(Node::Boolean)
                    .ok(),
                Some(ValidatorTypes::IntegerValidator(..)) => {
                    row.try_get::<i64, usize>(col_index).map(Node::Integer).ok()
                }
                Some(ValidatorTypes::NumberValidator(..)) => row
                    .try_get::<f64, usize>(col_index)
                    .map(|num| Node::Number(Number(num)))
                    .ok(),
                Some(ValidatorTypes::StringValidator(..)) => row
                    .try_get::<String, usize>(col_index)
                    .map(Node::String)
                    .ok(),
                _ => row
                    .try_get::<String, usize>(col_index)
                    .ok()
                    .and_then(|json| serde_json::from_str(&json).ok()),
            };
            if let Some(value) = value {
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

/// Transform a Stencila [`Datatable`] to a Postgres table
///
/// This function follows the recommendation [here](https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts)
/// for doing bulk inserts to Postgres
async fn datatable_to_postgres(name: &str, datatable: Datatable, pool: &AnyPool) -> Result<()> {
    todo!();
    
    let mut sql = datatable_to_create_table(name, &datatable);
    Ok(())
}

/// Transform a Stencila [`Datatable`] to SQL table
async fn datatable_to_other(name: &str, datatable: Datatable, pool: &AnyPool) -> Result<()> {
    let cols = datatable.columns.len();
    let rows = datatable
        .columns
        .first()
        .map(|column| column.values.len())
        .unwrap_or(0);

    let mut sql = datatable_to_create_table(name, &datatable);

    if rows > 0 {
        sql += &format!("INSERT INTO \"{}\" VALUES\n", name);
        sql += &vec![format!(" ({})", vec!["?"; cols].join(", ")); rows].join(",\n");
        sql += ";";
    }

    let mut query = sqlx::query(&sql);
    for row in 0..rows {
        for col in 0..cols {
            let column = &datatable.columns[col];
            let node = &column.values[row];
            match node {
                Node::Null(..) => query = query.bind("null"),
                Node::Boolean(value) => query = query.bind(value),
                Node::Integer(value) => query = query.bind(value),
                Node::Number(value) => query = query.bind(value.0),
                Node::String(value) => query = query.bind(value),
                _ => query = query.bind(serde_json::to_string(node).unwrap_or_default()),
            }
        }
    }
    query.execute(pool).await?;

    Ok(())
}

/// Generate a `CREATE TABLE` query
fn datatable_to_create_table(name: &str, datatable: &Datatable) -> String {
    let columns = datatable
        .columns
        .iter()
        .map(|column| {
            let validator = column
                .validator
                .as_deref()
                .and_then(|array_validator| array_validator.items_validator.clone());
            let datatype = match validator.as_deref() {
                Some(ValidatorTypes::BooleanValidator(..)) => "BOOLEAN",
                Some(ValidatorTypes::IntegerValidator(..)) => "INTEGER",
                Some(ValidatorTypes::NumberValidator(..)) => "REAL",
                Some(ValidatorTypes::StringValidator(..)) => "TEXT",
                _ => "JSON",
            };
            format!("{} {}", column.name, datatype)
        })
        .collect_vec()
        .join(", ");
    format!(
        "DROP TABLE IF EXISTS \"{}\";\nCREATE TABLE \"{}\"({});\n",
        name, name, columns
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{common::tokio, KernelTrait, stencila_schema::Primitive};
    use test_utils::assert_json_eq;

    #[tokio::test]
    async fn get_set_exec() -> Result<()> {
        let mut kernel = SqlKernel::new(&KernelSelector::default());

        if let Ok(..) = kernel.get("table_a").await {
            bail!("Expected an error because table not yet created")
        };

        match kernel.set("table_a", Node::String("A".to_string())).await {
            Ok(..) => bail!("Expected an error"),
            Err(error) => assert!(error
                .to_string()
                .contains("Only Datatables can be set as symbols")),
        };

        let rows = 5;
        let col_1 = DatatableColumn {
            name: "col_1".to_string(),
            validator: Some(Box::new(ArrayValidator {
                items_validator: Some(Box::new(ValidatorTypes::BooleanValidator(
                    BooleanValidator::default(),
                ))),
                ..Default::default()
            })),
            values: vec![Node::Boolean(true); rows],
            ..Default::default()
        };
        let col_2 = DatatableColumn {
            name: "col_2".to_string(),
            validator: Some(Box::new(ArrayValidator {
                items_validator: Some(Box::new(ValidatorTypes::IntegerValidator(
                    IntegerValidator::default(),
                ))),
                ..Default::default()
            })),
            values: (0..rows)
                .map(|index| Node::Integer(index as i64))
                .collect_vec(),
            ..Default::default()
        };
        let col_3 = DatatableColumn {
            name: "col_3".to_string(),
            validator: Some(Box::new(ArrayValidator {
                items_validator: Some(Box::new(ValidatorTypes::NumberValidator(
                    NumberValidator::default(),
                ))),
                ..Default::default()
            })),
            values: (0..rows)
                .map(|index| Node::Number(Number(index as f64)))
                .collect_vec(),
            ..Default::default()
        };
        let col_4 = DatatableColumn {
            name: "col_4".to_string(),
            validator: Some(Box::new(ArrayValidator {
                items_validator: Some(Box::new(ValidatorTypes::StringValidator(
                    StringValidator::default(),
                ))),
                ..Default::default()
            })),
            values: (0..rows)
                .map(|index| Node::String(format!("string-{}", index)))
                .collect_vec(),
            ..Default::default()
        };
        let col_5 = DatatableColumn {
            name: "col_5".to_string(),
            validator: None,
            values: (0..rows)
                .map(|index| Node::Array(vec![Primitive::Integer(index as i64)]))
                .collect_vec(),
            ..Default::default()
        };
        let datatable_a = Datatable {
            columns: vec![col_1, col_2, col_3, col_4, col_5],
            ..Default::default()
        };
        kernel
            .set("table_a", Node::Datatable(datatable_a.clone()))
            .await?;

        let table_a = kernel.get("table_a").await?;
        assert!(matches!(table_a, Node::Datatable(..)));
        assert_json_eq!(table_a, datatable_a);

        Ok(())
    }
}
