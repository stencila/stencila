use sqlx::{Column, Row, SqlitePool, TypeInfo};

use kernel::{
    common::{eyre::Result, itertools::Itertools, serde_json, tracing},
    stencila_schema::{
        ArrayValidator, BooleanValidator, Datatable, DatatableColumn, IntegerValidator, Node, Null,
        Number, NumberValidator, StringValidator, ValidatorTypes,
    },
};

/// Run a query in SQLite and return the result as a Stencila [`Datatable`]
pub async fn query_to_datatable(query: &str, pool: &SqlitePool) -> Result<Datatable> {
    // Run the query
    let rows = sqlx::query(query).fetch_all(pool).await?;

    // Get the names of the columns and transform their types into validators
    let columns = if let Some(row) = rows.first() {
        row.columns()
            .iter()
            .map(|column| {
                let name = column.name().to_string();
                let col_type = column.type_info().name().to_string();
                let validator = match col_type.as_str() {
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
                "BOOLEAN" => row
                    .try_get::<bool, usize>(col_index)
                    .map(Node::Boolean)
                    .ok(),
                "INTEGER" => row.try_get::<i64, usize>(col_index).map(Node::Integer).ok(),
                "REAL" => row
                    .try_get::<f64, usize>(col_index)
                    .map(|num| Node::Number(Number(num)))
                    .ok(),
                "TEXT" => row
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

/// Create a SQLite table from a Stencila [`Datatable`]
pub async fn table_from_datatable(
    name: &str,
    datatable: Datatable,
    pool: &SqlitePool,
) -> Result<()> {
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

    let cols = datatable.columns.len();

    let mut sql = format!("INSERT INTO \"{}\" VALUES\n", name);
    sql += &vec![format!(" ({})", vec!["?"; cols].join(", ")); rows].join(",\n");

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
