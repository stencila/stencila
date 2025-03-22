use kuzu::{Error, LogicalType, NodeVal, QueryResult, RelVal, Value};

use kernel::{
    common::{
        self,
        eyre::{Result, bail},
        once_cell::sync::Lazy,
        regex::Regex,
        serde::Serialize,
        serde_json::{self, json},
    },
    schema::*,
};

/// Create a Stencila [`ImageObject`] containing a Cytoscape.js graph from a Kuzu [`QueryResult`]
pub fn cytoscape_from_query_result(result: QueryResult) -> Result<ImageObject> {
    #[derive(Serialize)]
    #[serde(crate = "common::serde")]
    struct Element {
        data: Data,
    }

    #[derive(Serialize)]
    #[serde(untagged, crate = "common::serde")]
    enum Data {
        Node {
            id: String,
        },
        Edge {
            id: String,
            source: String,
            target: String,
        },
    }

    fn node(node: NodeVal) -> Element {
        Element {
            data: Data::Node {
                id: node.get_node_id().to_string(),
            },
        }
    }

    fn edge(rel: RelVal) -> Element {
        let source = rel.get_src_node().to_string();
        let target = rel.get_dst_node().to_string();
        Element {
            data: Data::Edge {
                id: [&source, ".", &target].concat(),
                source,
                target,
            },
        }
    }

    let mut elements = Vec::new();
    for row in result {
        for value in row {
            elements.push(match value {
                Value::Node(node_val) => node(node_val),
                Value::Rel(rel_val) => edge(rel_val),
                _ => continue,
            });
        }
    }

    let json = serde_json::to_string(&json!({
        "elements": elements,
        "layout": {
            "name": "cose"
        }
    }))?;

    Ok(ImageObject {
        content_url: json,
        media_type: Some("application/vnd.cytoscape.v3+json".into()),
        ..Default::default()
    })
}

/// Create a Stencila [`Datatable`] from a Kuzu [`QueryResult`]
pub fn datatable_from_query_result(result: QueryResult) -> Result<Datatable> {
    let mut columns: Vec<DatatableColumn> = result
        .get_column_names()
        .into_iter()
        .zip(result.get_column_data_types())
        .map(|(name, data_type)| DatatableColumn {
            name,
            validator: array_validator_from_logical_type(&data_type),
            values: Vec::new(),
            ..Default::default()
        })
        .collect();

    for row in result {
        for (col, value) in row.into_iter().enumerate() {
            let Some(column) = columns.get_mut(col) else {
                bail!("Invalid index");
            };

            let value = primitive_from_value(value);
            column.values.push(value);
        }
    }

    Ok(Datatable {
        columns,
        ..Default::default()
    })
}

/// Get the Stencila [`Validator`] corresponding to a Kuzu [`LogicalType`]
pub fn validator_from_logical_type(logical_type: &LogicalType) -> Option<Validator> {
    match logical_type {
        LogicalType::Bool => Some(Validator::BooleanValidator(BooleanValidator::default())),
        LogicalType::Int8
        | LogicalType::Int16
        | LogicalType::Int32
        | LogicalType::Int64
        | LogicalType::Int128 => Some(Validator::IntegerValidator(IntegerValidator::default())),
        LogicalType::UInt8 | LogicalType::UInt16 | LogicalType::UInt32 | LogicalType::UInt64 => {
            Some(Validator::IntegerValidator(IntegerValidator {
                minimum: Some(0.),
                ..Default::default()
            }))
        }
        LogicalType::Float | LogicalType::Double => {
            Some(Validator::NumberValidator(NumberValidator::default()))
        }
        LogicalType::String => Some(Validator::StringValidator(StringValidator::default())),
        _ => None,
    }
}

/// Get the Stencila [`ArrayValidator`] corresponding to a Kuzu column type
pub fn array_validator_from_logical_type(logical_type: &LogicalType) -> Option<ArrayValidator> {
    validator_from_logical_type(logical_type).map(|validator| ArrayValidator {
        items_validator: Some(Box::new(validator)),
        items_nullable: Some(true),
        ..Default::default()
    })
}

/// Create a Stencila [`Primitive`] from a Kuzu [`Value`]
pub fn primitive_from_value(value: Value) -> Primitive {
    match value {
        Value::Null(..) => Primitive::Null(Null),
        Value::Bool(value) => Primitive::Boolean(value),
        Value::Int8(value) => Primitive::Integer(value as i64),
        Value::Int16(value) => Primitive::Integer(value as i64),
        Value::Int32(value) => Primitive::Integer(value as i64),
        Value::Int64(value) => Primitive::Integer(value),
        Value::Int128(value) => Primitive::Integer(value as i64),
        Value::UInt8(value) => Primitive::Integer(value as i64),
        Value::UInt16(value) => Primitive::Integer(value as i64),
        Value::UInt32(value) => Primitive::Integer(value as i64),
        Value::UInt64(value) => Primitive::Integer(value as i64),
        Value::Float(value) => Primitive::Number(value as f64),
        Value::Double(value) => Primitive::Number(value),
        _ => Primitive::String(value.to_string()),
    }
}

/// Create a Stencila [`ExecutionMessage`] from a Kuzu [`Error`]
pub fn execution_message_from_error(error: Error, line_offset: usize) -> ExecutionMessage {
    let error = error
        .to_string()
        .replace("Query execution failed:", "")
        .replace("Binder exception:", "")
        .trim()
        .to_string();

    static PARSER_EXC_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?ms)\(line: (\d+), offset: (\d+)\).*?"(.*?)""#).expect("invalid regex")
    });

    let mut code_location = None;
    let message = if let Some(rest) = error.strip_prefix("Parser exception:") {
        match PARSER_EXC_REGEX.captures(&rest) {
            Some(captures) => {
                code_location = Some(CodeLocation {
                    start_line: captures[1]
                        .parse()
                        .ok()
                        .map(|line: u64| line + line_offset as u64 - 1),
                    start_column: captures[2].parse().ok(),
                    ..Default::default()
                });

                let rest = captures[3].trim();
                if rest.is_empty() {
                    "Syntax error".to_string()
                } else {
                    ["Syntax error: ", rest].concat()
                }
            }
            None => ["Syntax error: ", rest].concat(),
        }
    } else {
        error
    };

    ExecutionMessage {
        level: MessageLevel::Error,
        message,
        code_location,
        ..Default::default()
    }
}
