use std::collections::HashMap;

use sqlx::{postgres::PgArguments, Arguments, Column, PgPool, Row, TypeInfo};

use kernel::{
    common::{eyre::Result, itertools::Itertools, regex::Captures, serde_json, tracing},
    stencila_schema::{
        ArrayValidator, BooleanValidator, Datatable, DatatableColumn, IntegerValidator, Node, Null,
        Number, NumberValidator, StringValidator, ValidatorTypes,
    },
};

use crate::BINDING_REGEX;

/// Bind parameters to an SQL statement based on name
fn bind(sql: &str, parameters: &HashMap<String, Node>) -> (String, PgArguments) {
    let mut count = 0;
    let mut arguments = PgArguments::default();
    let sql = BINDING_REGEX.replace_all(sql, |captures: &Captures| {
        let name = captures[1].to_string();
        let value = parameters.get(&name).unwrap();
        match value {
            Node::Boolean(value) => arguments.add(value),
            Node::Integer(value) => arguments.add(value),
            Node::Number(value) => arguments.add(value.0),
            Node::String(value) => arguments.add(value),
            _ => arguments.add(serde_json::to_value(&value).unwrap_or(serde_json::Value::Null)),
        };
        count += 1;
        ["$", &count.to_string()].concat()
    });
    (sql.to_string(), arguments)
}

/// Execute an SQL statement in Postgres
///
/// Only returns a `Datatable` for convenience elsewhere in the code
pub async fn execute_statement(
    sql: &str,
    parameters: &HashMap<String, Node>,
    pool: &PgPool,
) -> Result<Datatable> {
    let (sql, args) = bind(sql, parameters);
    sqlx::query_with(&sql, args).execute(pool).await?;
    Ok(Datatable::default())
}

/// Run a query in Postgres and return the result as a Stencila [`Datatable`]
pub async fn query_to_datatable(
    query: &str,
    parameters: &HashMap<String, Node>,
    pool: &PgPool,
) -> Result<Datatable> {
    // Run the query
    let (sql, args) = bind(query, parameters);
    let rows = sqlx::query_with(&sql, args).fetch_all(pool).await?;

    // Get the names of the columns and transform their types into validators
    let columns = if let Some(row) = rows.first() {
        row.columns()
            .iter()
            .map(|column| {
                let name = column.name().to_string();
                let col_type = column.type_info().name().to_string();
                let validator = match col_type.as_str() {
                    "BOOL" => Some(ValidatorTypes::BooleanValidator(BooleanValidator::default())),
                    "INT4" | "INT8" => {
                        Some(ValidatorTypes::IntegerValidator(IntegerValidator::default()))
                    }
                    "FLOAT4" | "FLOAT8" => {
                        Some(ValidatorTypes::NumberValidator(NumberValidator::default()))
                    }
                    "TEXT" => Some(ValidatorTypes::StringValidator(StringValidator::default())),
                    "JSON" => None,
                    _ => {
                        tracing::error!("Unhandled column type: {}", col_type);
                        None
                    }
                };
                (name, col_type, validator)
            })
            .collect()
    } else {
        Vec::new()
    };

    // Pre-allocate an vector of the size needed to hold all values and insert them in
    // column-first order
    let rows_len = rows.len();
    let mut values: Vec<Node> = vec![Node::Null(Null {}); columns.len() * rows_len];
    for (row_index, row) in rows.into_iter().enumerate() {
        for (col_index, (_name, col_type, ..)) in columns.iter().enumerate() {
            let position = col_index * rows_len + row_index;
            let value = match col_type.as_str() {
                "BOOL" => row
                    .try_get::<bool, usize>(col_index)
                    .map(Node::Boolean)
                    .ok(),
                "INT4" => row
                    .try_get::<i32, usize>(col_index)
                    .map(|int| Node::Integer(int as i64))
                    .ok(),
                "INT8" => row.try_get::<i64, usize>(col_index).map(Node::Integer).ok(),
                "FLOAT4" => row
                    .try_get::<f32, usize>(col_index)
                    .map(|num| Node::Number(Number(num as f64)))
                    .ok(),
                "FLOAT8" => row
                    .try_get::<f64, usize>(col_index)
                    .map(|num| Node::Number(Number(num)))
                    .ok(),
                "TEXT" => row
                    .try_get::<String, usize>(col_index)
                    .map(Node::String)
                    .ok(),
                "JSON" => row
                    .try_get::<serde_json::Value, usize>(col_index)
                    .ok()
                    .and_then(|value| serde_json::from_value(value).ok()),
                _ => row
                    .try_get_unchecked::<String, usize>(col_index)
                    .ok()
                    .and_then(|json| serde_json::from_str(&json).ok()),
            };
            if let Some(value) = value {
                values[position] = value;
            }
        }
    }

    // Create datatable
    let columns = columns
        .into_iter()
        .map(|(name, _col_type, validator)| DatatableColumn {
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
    Ok(Datatable {
        columns,
        ..Default::default()
    })
}

/// Create a Postgres table from a Stencila [`Datatable`]
///
/// This function follows the recommendation [here](https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts)
/// for doing bulk inserts into Postgres tables.
pub async fn table_from_datatable(name: &str, datatable: Datatable, pool: &PgPool) -> Result<()> {
    sqlx::query(&format!("DROP TABLE IF EXISTS \"{}\"", name))
        .execute(pool)
        .await?;

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
    sqlx::query(&format!("CREATE TABLE \"{}\"({});\n", name, columns))
        .execute(pool)
        .await?;

    let rows = datatable
        .columns
        .first()
        .map(|column| column.values.len())
        .unwrap_or(0);
    if rows == 0 {
        return Ok(());
    }

    let validators = datatable
        .columns
        .iter()
        .map(|column| {
            column
                .validator
                .as_deref()
                .and_then(|array_validator| array_validator.items_validator.as_deref())
        })
        .collect_vec();

    let bindings = validators
        .iter()
        .enumerate()
        .map(|(index, validator)| {
            let datatype = match validator {
                Some(ValidatorTypes::BooleanValidator(..)) => "BOOLEAN",
                Some(ValidatorTypes::IntegerValidator(..)) => "INTEGER",
                Some(ValidatorTypes::NumberValidator(..)) => "REAL",
                Some(ValidatorTypes::StringValidator(..)) => "TEXT",
                _ => "JSON",
            };
            format!("${}::{}[]", index + 1, datatype)
        })
        .collect_vec()
        .join(", ");
    let sql = format!(
        "INSERT INTO \"{}\"\nSELECT * FROM UNNEST({})",
        name, bindings
    );

    let mut query = sqlx::query(&sql);
    for (index, validator) in validators.iter().enumerate() {
        let values = datatable.columns[index].values.iter();
        match validator {
            Some(ValidatorTypes::BooleanValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Node::Boolean(value) => Some(*value),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::IntegerValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Node::Integer(value) => Some(*value),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::NumberValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Node::Number(value) => Some(value.0),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::StringValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Node::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            _ => {
                let values = values
                    .map(|node| serde_json::to_string(node).unwrap_or_default())
                    .collect_vec();
                query = query.bind(values);
            }
        }
    }

    query.execute(pool).await?;

    Ok(())
}
