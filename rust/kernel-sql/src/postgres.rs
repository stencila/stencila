use std::collections::HashMap;

use events::publish;
use sqlx::{
    postgres::{PgArguments, PgListener},
    Arguments, Column, PgPool, Row, TypeInfo,
};

use kernel::{
    common::{
        chrono,
        eyre::{bail, Result},
        itertools::Itertools,
        regex::Captures,
        serde_json, tokio, tracing,
    },
    stencila_schema::{
        ArrayValidator, BooleanValidator, Datatable, DatatableColumn, Date, DateTime,
        DateValidator, EnumValidator, IntegerValidator, Node, Null, Number, NumberValidator,
        Parameter, Primitive, StringValidator, Time, TimeValidator, ValidatorTypes,
    },
};

use crate::{
    common::{create_table_from_datatable, datatable_rows_cols},
    WatchedTables, BINDING_REGEX,
};

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
            _ => arguments.add(serde_json::to_value(value).unwrap_or(serde_json::Value::Null)),
        };
        count += 1;
        ["$", &count.to_string()].concat()
    });
    (sql.to_string(), arguments)
}

/// Execute an SQL statement in Postgres
pub async fn execute_statement(
    sql: &str,
    parameters: &HashMap<String, Node>,
    pool: &PgPool,
) -> Result<()> {
    let (sql, args) = bind(sql, parameters);
    sqlx::query_with(&sql, args).execute(pool).await?;
    Ok(())
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
                    "DATE" => Some(ValidatorTypes::DateValidator(DateValidator::default())),
                    "TIME" => Some(ValidatorTypes::TimeValidator(TimeValidator::default())),
                    "TIMESTAMP" => Some(ValidatorTypes::DateTimeValidator(
                        kernel::stencila_schema::DateTimeValidator::default(),
                    )),
                    "JSON" => None,
                    _ => {
                        tracing::warn!(
                            "Unhandled column type, will have no validator: {}",
                            col_type
                        );
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
    let mut values: Vec<Primitive> = vec![Primitive::Null(Null {}); columns.len() * rows_len];
    for (row_index, row) in rows.into_iter().enumerate() {
        for (col_index, (_name, col_type, ..)) in columns.iter().enumerate() {
            let position = col_index * rows_len + row_index;
            let value = match col_type.as_str() {
                "BOOL" => row
                    .try_get::<bool, usize>(col_index)
                    .map(Primitive::Boolean)
                    .ok(),
                "INT4" => row
                    .try_get::<i32, usize>(col_index)
                    .map(|int| Primitive::Integer(int as i64))
                    .ok(),
                "INT8" => row
                    .try_get::<i64, usize>(col_index)
                    .map(Primitive::Integer)
                    .ok(),
                "FLOAT4" => row
                    .try_get::<f32, usize>(col_index)
                    .map(|num| Primitive::Number(Number(num as f64)))
                    .ok(),
                "FLOAT8" => row
                    .try_get::<f64, usize>(col_index)
                    .map(|num| Primitive::Number(Number(num)))
                    .ok(),
                "TEXT" => row
                    .try_get::<String, usize>(col_index)
                    .map(Primitive::String)
                    .ok(),
                "DATE" => row
                    .try_get::<chrono::NaiveDate, usize>(col_index)
                    .map(|date| Primitive::Date(Date::from(date)))
                    .ok(),
                "TIME" => row
                    .try_get::<chrono::NaiveTime, usize>(col_index)
                    .map(|time| Primitive::Time(Time::from(time)))
                    .ok(),
                "TIMESTAMP" => row
                    .try_get::<chrono::NaiveDateTime, usize>(col_index)
                    .map(|datetime| Primitive::DateTime(DateTime::from(datetime)))
                    .ok(),
                "JSON" => row
                    .try_get::<serde_json::Value, usize>(col_index)
                    .ok()
                    .and_then(|value| serde_json::from_value(value).ok()),
                _ => row
                    .try_get_unchecked::<String, usize>(col_index)
                    .ok()
                    .and_then(|string| {
                        serde_json::from_str(&string).unwrap_or(Some(Primitive::String(string)))
                    }),
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
                    // Assume true (the default) because unable to get the columns NOT NULL easily
                    items_nullable: true,
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
    let (rows, cols) = datatable_rows_cols(&datatable);
    if cols == 0 {
        return Ok(());
    }

    sqlx::query(&format!("DROP TABLE IF EXISTS \"{}\"", name))
        .execute(pool)
        .await?;

    let sql = create_table_from_datatable(name, &datatable, false);
    sqlx::query(&sql).execute(pool).await?;

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
                Some(ValidatorTypes::DateValidator(..)) => "DATE",
                Some(ValidatorTypes::TimeValidator(..)) => "TIME",
                Some(ValidatorTypes::DateTimeValidator(..)) => "TIMESTAMP",
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
                        Primitive::Boolean(value) => Some(*value),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::IntegerValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Primitive::Integer(value) => Some(*value),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::NumberValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Primitive::Number(value) => Some(value.0),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::StringValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Primitive::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::DateValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Primitive::Date(value) => Some(value.value.clone()),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::TimeValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Primitive::Time(value) => Some(value.value.clone()),
                        _ => None,
                    })
                    .collect_vec();
                query = query.bind(values);
            }
            Some(ValidatorTypes::DateTimeValidator(..)) => {
                let values = values
                    .map(|node| match node {
                        Primitive::DateTime(value) => Some(value.value.clone()),
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

/**
 * Derive parameters from the columns of a Postgres table
 */
pub async fn table_to_parameters(
    url: &str,
    pool: &PgPool,
    table: &str,
    schema: Option<&str>,
) -> Result<Vec<Parameter>> {
    // Get table metadata
    let schema = schema.unwrap_or("public");
    let columns = sqlx::query(
        r#"
        select column_name, data_type, udt_schema, udt_name, column_default
        from information_schema.columns
        where table_schema = $1 and table_name = $2;
        "#,
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await?;

    if columns.is_empty() {
        bail!(
            "Table `{}` does not appear to exists in schema `{}` of Postgres database `{}`",
            table,
            schema,
            url
        )
    }

    // Get checks associated with the table
    let checks = sqlx::query(
        r#"
        select column_name, check_clause
        from information_schema.check_constraints as cc
        left join information_schema.constraint_column_usage as ccu
        on cc.constraint_name = ccu.constraint_name
        where table_schema = $1 and table_name = $2;
        "#,
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await?;
    let checks: HashMap<String, String> = checks
        .into_iter()
        .map(|check| {
            (
                check.get::<String, &str>("column_name"),
                check.get::<String, &str>("check_clause"),
            )
        })
        .collect();

    // Generate the SQL for the table.
    // Also record details of columns with user defined data types
    use std::fmt::Write;
    let mut sql = format!("CREATE TABLE {} (\n", table);
    let mut enum_columns: Vec<(String, String)> = Vec::new();
    for column in columns {
        let column_name: String = column.get("column_name");
        let data_type: String = column.get("data_type");
        let column_default: Option<String> = column.get("column_default");
        writeln!(
            &mut sql,
            "  {column_name} {data_type} {default} {checks},",
            default = column_default
                .map(|value| ["DEFAULT ", &value].concat())
                .unwrap_or_default(),
            checks = checks
                .get(&column_name)
                .map(|value| ["CHECK ", value].concat())
                .unwrap_or_default()
        )?;

        if data_type == "USER-DEFINED" {
            enum_columns.push((column_name, column.get("udt_name")));
        }
    }
    sql += ");";

    // Parse the SQL to get the parameters (including checks)
    let mut parameters = parser_sql::SqlParser::derive_parameters(table, &sql);

    // Add missing validators for enum columns
    for (column_name, data_type) in enum_columns {
        if let Some(values) = sqlx::query(&format!(
            r#"select enum_range(null::{})::text[]"#,
            data_type
        ))
        .fetch_optional(pool)
        .await?
        {
            let values = values
                .get::<Vec<String>, usize>(0)
                .into_iter()
                .map(Node::String)
                .collect_vec();
            for parameter in &mut parameters {
                if parameter.name == column_name {
                    parameter.validator =
                        Some(Box::new(ValidatorTypes::EnumValidator(EnumValidator {
                            values,
                            ..Default::default()
                        })));
                    break;
                }
            }
        }
    }

    Ok(parameters)
}

/**
 * Derive a parameter from a column in a Postgres table
 */
pub async fn column_to_parameter(
    url: &str,
    pool: &PgPool,
    column: &str,
    table: &str,
    schema: Option<&str>,
) -> Result<Parameter> {
    let parameter = table_to_parameters(url, pool, table, schema)
        .await?
        .into_iter()
        .find(|parameter| parameter.name == [table, "_", column].concat());

    let schema = schema.unwrap_or("public");
    match parameter {
        Some(parameter) => Ok(parameter),
        None => {
            bail!(
                "Column `{}` could not be found in table `{}` of schema `{}` of Postgres database `{}`",
                column, table, schema, url
            )
        }
    }
}

/// Start a background task to listen for notifications of changes to tables
pub async fn watch(url: &str, pool: &PgPool, watches: WatchedTables) -> Result<()> {
    sqlx::query(
        "
        CREATE OR REPLACE FUNCTION stencila_resource_change_trigger()
        RETURNS trigger
        LANGUAGE plpgsql
        AS $trigger$
        BEGIN
            PERFORM pg_notify(
            'stencila_resource_change',
            json_build_object(
                'time', CURRENT_TIMESTAMP,
                'action', LOWER(TG_OP),
                'schema', TG_TABLE_SCHEMA,
                'table', TG_TABLE_NAME
            )::text
            );
            RETURN NULL;
        END;
        $trigger$;
        ",
    )
    .execute(pool)
    .await?;

    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen("stencila_resource_change").await?;

    let url = url.to_string();
    tokio::spawn(async move {
        while let Ok(notification) = listener.recv().await {
            let watches = watches.read().await;
            if watches.is_empty() {
                continue;
            }

            let json = notification.payload();
            let mut event: HashMap<String, String> = match serde_json::from_str(json) {
                Ok(event) => event,
                Err(error) => {
                    tracing::error!("While deserializing Postgres notification event: {}", error);
                    continue;
                }
            };

            let schema = event
                .remove("schema")
                .unwrap_or_else(|| "public".to_string());

            let name = match event.remove("table") {
                Some(table) => table,
                None => {
                    tracing::error!("Postgres notification event has no `table` field: {}", json);
                    continue;
                }
            };

            if !watches.contains(&name) {
                continue;
            }

            publish(
                &["databases:", &url, ":", &schema, ":", &name].concat(),
                "Updated",
            );
        }
    });

    Ok(())
}

/// Set up watches for a particular table
pub async fn watch_table(table: &str, pool: &PgPool) -> Result<()> {
    sqlx::query(&format!(
        r#"
        CREATE OR REPLACE TRIGGER "stencila_resource_change_{table}"
        AFTER INSERT OR UPDATE OR DELETE ON "{table}"
        EXECUTE PROCEDURE stencila_resource_change_trigger();
        "#
    ))
    .execute(pool)
    .await?;

    Ok(())
}

/// Set up watches for `@all` tables in a schema
pub async fn watch_all(schema: Option<&String>, pool: &PgPool) -> Result<Vec<String>> {
    let schema = schema.map_or_else(|| "public".to_string(), String::from);

    let tables = sqlx::query(
        r#"
        SELECT table_name FROM information_schema.tables
        WHERE table_schema = $1
        "#,
    )
    .bind(schema)
    .fetch_all(pool)
    .await?;

    let mut names = Vec::with_capacity(tables.len());
    for table in tables {
        let name = table.get_unchecked("table_name");
        watch_table(name, pool).await?;
        names.push(name.to_owned());
    }

    Ok(names)
}
