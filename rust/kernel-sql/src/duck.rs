use std::{collections::HashMap, convert::TryInto};

use arrow::{
    array,
    datatypes::{self, DataType},
};
use duckdb::{params_from_iter, types::ToSqlOutput, Connection, ToSql};

use kernel::{
    common::{
        chrono,
        eyre::{bail, Result},
        itertools::Itertools,
        regex::Captures,
        tokio::sync::{mpsc, Mutex},
        tracing,
    },
    graph_triples::ResourceChange,
    stencila_schema::{
        ArrayValidator, BooleanValidator, Datatable, DatatableColumn, Date, DateTime,
        DateValidator, Duration, DurationValidator, EnumValidator, IntegerValidator, Node, Null,
        Number, NumberValidator, Primitive, StringValidator, Time, TimeUnit, TimeValidator,
        Timestamp, TimestampValidator, ValidatorTypes,
    },
};

use crate::{WatchedTables, BINDING_REGEX};

/// A connection "pool" to a DuckDB database
///
/// Used as an analog of `PgPool` and `SqlitePool` which are provided by `sqlx`.
/// Named `DuckPond` because...I couldn't resist.
#[derive(Debug)]
pub struct DuckPond {
    path: String,

    connection: Mutex<Connection>,
}

impl DuckPond {
    pub fn connect(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            connection: Mutex::new(Self::open(path)),
        }
    }

    fn open(path: &str) -> Connection {
        let result = match path {
            ":memory:" => Connection::open_in_memory(),
            _ => Connection::open(path),
        };
        match result {
            Ok(connection) => connection,
            Err(error) => {
                tracing::error!("While attempting to open DuckDB `{}`: {}", path, error);
                Connection::open_in_memory().expect("Unable to open in-memory DuckDB")
            }
        }
    }
}

impl Clone for DuckPond {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            connection: Mutex::new(Self::open(&self.path)),
        }
    }
}

/// Convert parameters to SQL string
/// 
/// Unfortunately due to lifetimes and `ToSqlOutput` we seem to need to do this before calling `bind`
fn to_sql(parameters: &HashMap<String, Node>) -> HashMap<String, Node> {
    parameters
        .iter()
        .map(|(name, node)| {
            (
                name.to_owned(),
                match node {
                    Node::Date(date) => Node::String(date.to_sql()),
                    Node::Time(time) => Node::String(time.to_sql()),
                    Node::DateTime(datetime) => Node::String(datetime.to_sql()),
                    Node::Timestamp(timestamp) => {
                        Node::String(timestamp.to_sql().unwrap_or_default())
                    }
                    Node::Duration(duration) => Node::String(duration.to_sql()),
                    _ => node.clone(),
                },
            )
        })
        .collect()
}

/// Bind parameters to an SQL statement based on name
fn bind<'p>(
    sql: &str,
    parameters: &'p HashMap<String, Node>,
) -> Result<(String, Vec<ToSqlOutput<'p>>)> {
    let mut count = 0;
    let mut used = Vec::new();
    let mut missing = Vec::new();
    let sql = BINDING_REGEX.replace_all(sql, |captures: &Captures| {
        count += 1;
        let name = captures[1].to_string();
        let placeholder = match &parameters.get(&name) {
            Some(node) => {
                let cast = match node {
                    // In SQL, cast strings to types as necessary
                    Node::Date(..) => "::DATE",
                    Node::Time(..) => "::TIME",
                    Node::DateTime(..) => "::TIMESTAMP",
                    Node::Timestamp(..) => "::TIMESTAMP",
                    Node::Duration(..) => "::INTERVAL",
                    _ => "",
                };
                used.push(name);
                let index = count.to_string();
                ["?", &index, cast].concat()
            }
            None => {
                missing.push(name);
                String::new()
            }
        };
        placeholder
    });

    if !missing.is_empty() {
        bail!("Parameters in SQL were not found: {}", missing.join(", "));
    }

    let mut params = Vec::new();
    for name in used {
        let node = parameters.get(&name).unwrap();
        let to_sql_output = match node {
            Node::Boolean(value) => value.to_sql()?,
            Node::Integer(value) => value.to_sql()?,
            Node::Number(value) => value.0.to_sql()?,
            Node::String(value) => value.to_sql()?,
            Node::Date(Date { value, .. })
            | Node::Time(Time { value, .. })
            | Node::DateTime(DateTime { value, .. }) => value.to_sql()?,
            _ => bail!("Unhandled parameter type: {}", node.as_ref()),
        };
        params.push(to_sql_output);
    }

    Ok((sql.to_string(), params))
}

/// Execute an SQL statement in DuckDB
///
/// Only returns a `Datatable` for convenience elsewhere in the code
pub async fn execute_statement(
    sql: &str,
    parameters: &HashMap<String, Node>,
    pond: &DuckPond,
) -> Result<Datatable> {
    let connection = pond.connection.lock().await;

    let parameters = to_sql(parameters);
    let (sql, params) = bind(sql, &parameters)?;
    let mut statement = connection.prepare(&sql)?;

    statement.execute(params_from_iter(params))?;

    Ok(Datatable::default())
}

/// Run a query in SQLite and return the result as a Stencila [`Datatable`]
pub async fn query_to_datatable(
    sql: &str,
    parameters: &HashMap<String, Node>,
    pond: &DuckPond,
) -> Result<Datatable> {
    let connection = pond.connection.lock().await;

    let parameters = to_sql(parameters);
    let (sql, params) = bind(sql, &parameters)?;
    let mut statement = connection.prepare(&sql)?;

    let params = params_from_iter(params);
    let mut arrow = statement.query_arrow(params)?;

    // TODO: Do not use first but rather append values together and assert that
    // schema is the same
    let record_set = match arrow.next() {
        Some(record_set) => record_set,
        None => return Ok(Datatable::default()),
    };

    let schema = record_set.schema();
    let cols_num = record_set.num_columns();

    // Macros for casting rows Arrow arrays into `Vec<Node>`.
    // Note that these create `Null` if the value is `None`.

    const NULL: Primitive = Primitive::Null(Null {});
    let epoch = chrono::NaiveDate::from_ymd(1970, 1, 1);

    macro_rules! cast_to_ints {
        ($array_type:ty, $values:expr) => {
            $values
                .downcast_ref::<$array_type>()
                .expect("Should cast to expected type")
                .iter()
                .map(|value| value.map_or_else(|| NULL, |value| Primitive::Integer(value as i64)))
                .collect()
        };
    }

    macro_rules! cast_to_ints_u64 {
        ($array_type:ty, $values:expr) => {
            $values
                .downcast_ref::<$array_type>()
                .expect("Should cast to expected type")
                .iter()
                .map(|value| {
                    value.map_or_else(
                        || NULL,
                        |value| {
                            let value = match value > 0 {
                                true => value.try_into().unwrap_or(i64::MAX),
                                false => value.try_into().unwrap_or(i64::MIN),
                            };
                            Primitive::Integer(value)
                        },
                    )
                })
                .collect()
        };
    }

    macro_rules! cast_to_nums_f16 {
        ($array_type:ty, $values:expr) => {
            $values
                .downcast_ref::<$array_type>()
                .expect("Should cast to expected type")
                .iter()
                .map(|value| {
                    value.map_or_else(|| NULL, |value| Primitive::Number(Number(value.to_f64())))
                })
                .collect()
        };
    }

    macro_rules! cast_to_nums {
        ($array_type:ty, $values:expr) => {
            $values
                .downcast_ref::<$array_type>()
                .expect("Should cast to expected type")
                .iter()
                .map(|value| {
                    value.map_or_else(|| NULL, |value| Primitive::Number(Number(value as f64)))
                })
                .collect()
        };
    }

    macro_rules! cast_to_strings {
        ($array_type:ty, $values:expr) => {
            $values
                .downcast_ref::<$array_type>()
                .expect("Should cast to expected type")
                .iter()
                .map(|value| {
                    value.map_or_else(|| NULL, |value| Primitive::String(value.to_string()))
                })
                .collect()
        };
    }

    // Get the names of the columns and transform their types into validators
    let mut columns = Vec::with_capacity(cols_num);
    for index in 0..cols_num {
        let field = schema.field(index);
        let name = field.name().to_string();
        let values = record_set.column(index).as_any();

        let data_type = field.data_type();
        let items_nullable = field.is_nullable();
        let matched = match data_type {
            DataType::Boolean => Some((
                ValidatorTypes::BooleanValidator(BooleanValidator::default()),
                values
                    .downcast_ref::<array::BooleanArray>()
                    .expect("Should cast to expected type")
                    .iter()
                    .map(|value| value.map_or_else(|| NULL, Primitive::Boolean))
                    .collect(),
            )),
            DataType::Dictionary(key_type, value_type)
                if *key_type == Box::new(DataType::UInt8)
                    && *value_type == Box::new(DataType::Utf8) =>
            {
                let dict = values
                    .downcast_ref::<array::DictionaryArray<datatypes::UInt8Type>>()
                    .expect("Should cast to expected type");
                let enum_values = dict
                    .values()
                    .as_any()
                    .downcast_ref::<array::StringArray>()
                    .expect("Should cast to expected type")
                    .into_iter()
                    .flatten() // Should not have any nulls
                    .map(|value| value.to_string())
                    .collect_vec();
                let values = dict
                    .keys()
                    .into_iter()
                    .map(|key| {
                        key.map_or_else(
                            || NULL,
                            |key| Primitive::String(enum_values[key as usize].to_string()),
                        )
                    })
                    .collect_vec();
                Some((
                    ValidatorTypes::EnumValidator(EnumValidator {
                        values: enum_values
                            .iter()
                            .map(|value| Node::String(value.to_owned()))
                            .collect(),
                        ..Default::default()
                    }),
                    values,
                ))
            }
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => Some((
                ValidatorTypes::IntegerValidator(IntegerValidator::default()),
                match data_type {
                    DataType::Int8 => cast_to_ints!(array::Int8Array, values),
                    DataType::Int16 => cast_to_ints!(array::Int16Array, values),
                    DataType::Int32 => cast_to_ints!(array::Int32Array, values),
                    DataType::Int64 => cast_to_ints!(array::Int64Array, values),
                    _ => unreachable!(),
                },
            )),
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => Some((
                ValidatorTypes::IntegerValidator(IntegerValidator {
                    minimum: Some(Number(0f64)),
                    ..Default::default()
                }),
                match data_type {
                    DataType::UInt8 => cast_to_ints!(array::UInt8Array, values),
                    DataType::UInt16 => cast_to_ints!(array::UInt16Array, values),
                    DataType::UInt32 => cast_to_ints!(array::UInt32Array, values),
                    DataType::UInt64 => cast_to_ints_u64!(array::UInt64Array, values),
                    _ => unreachable!(),
                },
            )),
            DataType::Float16 | DataType::Float32 | DataType::Float64 => Some((
                ValidatorTypes::NumberValidator(NumberValidator::default()),
                match data_type {
                    DataType::Float16 => cast_to_nums_f16!(array::Float16Array, values),
                    DataType::Float32 => cast_to_nums!(array::Float32Array, values),
                    DataType::Float64 => cast_to_nums!(array::Float64Array, values),
                    _ => unreachable!(),
                },
            )),
            DataType::Decimal128(_precision, scale) if *scale == 0 => Some((
                ValidatorTypes::IntegerValidator(IntegerValidator::default()),
                values
                    .downcast_ref::<array::Decimal128Array>()
                    .expect("Should cast to expected type")
                    .iter()
                    .map(|value| {
                        value.map_or_else(
                            || NULL,
                            |value| {
                                let value = value.as_i128();
                                let value = match value > 0 {
                                    true => value.try_into().unwrap_or(i64::MAX),
                                    false => value.try_into().unwrap_or(i64::MIN),
                                };
                                Primitive::Integer(value)
                            },
                        )
                    })
                    .collect(),
            )),
            DataType::Decimal128(_precision, scale) => Some((
                ValidatorTypes::NumberValidator(NumberValidator::default()),
                values
                    .downcast_ref::<array::Decimal128Array>()
                    .expect("Should cast to expected type")
                    .iter()
                    .map(|value| {
                        value.map_or_else(
                            || NULL,
                            |value| {
                                Primitive::Number(Number(
                                    value.as_i128() as f64 / 10f64.powf(*scale as f64),
                                ))
                            },
                        )
                    })
                    .collect(),
            )),
            DataType::Utf8 | DataType::LargeUtf8 => Some((
                ValidatorTypes::StringValidator(StringValidator::default()),
                match data_type {
                    DataType::Utf8 => cast_to_strings!(array::StringArray, values),
                    DataType::LargeUtf8 => cast_to_strings!(array::LargeStringArray, values),
                    _ => unreachable!(),
                },
            )),
            DataType::Date32 => Some((
                ValidatorTypes::DateValidator(DateValidator::default()),
                values
                    .downcast_ref::<array::Date32Array>()
                    .expect("Should cast to expected type")
                    .iter()
                    .map(|value| {
                        value.map_or_else(
                            || NULL,
                            |value| match epoch
                                .checked_add_signed(chrono::Duration::days(value as i64))
                            {
                                Some(date) => Primitive::Date(Date::from(date)),
                                None => NULL,
                            },
                        )
                    })
                    .collect(),
            )),
            DataType::Time64(datatypes::TimeUnit::Microsecond) => Some((
                ValidatorTypes::TimeValidator(TimeValidator::default()),
                values
                    .downcast_ref::<array::Time64MicrosecondArray>()
                    .expect("Should cast to expected type")
                    .iter()
                    .map(|value| {
                        value.map_or_else(
                            || NULL,
                            |value| {
                                let secs = (value / 1000000) as u32;
                                let nanos = (value % 1000000) as u32;
                                match chrono::NaiveTime::from_num_seconds_from_midnight_opt(
                                    secs, nanos,
                                ) {
                                    Some(time) => Primitive::Time(Time::from(time)),
                                    None => NULL,
                                }
                            },
                        )
                    })
                    .collect(),
            )),
            DataType::Timestamp(datatypes::TimeUnit::Microsecond, ..) => Some((
                ValidatorTypes::TimestampValidator(TimestampValidator::default()),
                values
                    .downcast_ref::<array::TimestampMicrosecondArray>()
                    .expect("Should cast to expected type")
                    .iter()
                    .map(|value| {
                        value.map_or_else(
                            || NULL,
                            |value| {
                                Primitive::Timestamp(Timestamp {
                                    value,
                                    time_unit: TimeUnit::Microsecond,
                                    ..Default::default()
                                })
                            },
                        )
                    })
                    .collect(),
            )),
            DataType::Duration(datatypes::TimeUnit::Millisecond) => Some((
                ValidatorTypes::DurationValidator(DurationValidator::default()),
                values
                    .downcast_ref::<array::DurationMillisecondArray>()
                    .expect("Should cast to expected type")
                    .iter()
                    .map(|value| {
                        value.map_or_else(
                            || NULL,
                            |value| {
                                Primitive::Duration(Duration {
                                    value,
                                    time_unit: TimeUnit::Millisecond,
                                    ..Default::default()
                                })
                            },
                        )
                    })
                    .collect(),
            )),
            _ => {
                tracing::debug!("Unhandled DuckDB column type: {}", data_type);
                None
            }
        };

        if let Some((validator, values)) = matched {
            let column = DatatableColumn {
                name,
                validator: Some(Box::new(ArrayValidator {
                    items_validator: Some(Box::new(validator)),
                    items_nullable,
                    ..Default::default()
                })),
                values,
                ..Default::default()
            };
            columns.push(column)
        }
    }

    Ok(Datatable {
        columns,
        ..Default::default()
    })
}

/// Create a SQLite table from a Stencila [`Datatable`]
pub async fn table_from_datatable(
    _name: &str,
    _datatable: Datatable,
    _pond: &DuckPond,
) -> Result<()> {
    bail!("Converting a Datatable to a DuckDB table is not yet implemented")
}

/// Start a background task to listen for notifications of changes to tables
//
/// At present DuckDB does not support triggers or notifications so table
/// watching can not be supported.
pub async fn watch(
    _url: &str,
    _pond: &DuckPond,
    _watches: WatchedTables,
    _sender: mpsc::Sender<ResourceChange>,
) -> Result<()> {
    bail!("Table watches are not supported for DuckDB databases")
}

/// Set up watches for a particular table
pub async fn watch_table(_table: &str, _pond: &DuckPond) -> Result<()> {
    bail!("Table watches are not supported for DuckDB databases")
}

/// Set up watches for `@all` tables
pub async fn watch_all(_schema: Option<&String>, _pond: &DuckPond) -> Result<Vec<String>> {
    bail!("Table watches are not supported for DuckDB databases")
}
