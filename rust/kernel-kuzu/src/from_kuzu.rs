use std::collections::BTreeSet;

use kuzu::{Error, LogicalType, NodeVal, QueryResult, RelVal, Value};

use kernel::{
    common::{
        self,
        eyre::{Result, bail},
        indexmap::IndexMap,
        once_cell::sync::Lazy,
        regex::Regex,
        serde::Serialize,
        serde_json::{self, json},
    },
    schema::*,
};

/// Create a Stencila [`Node`] from a Kuzu [`QueryResult`]
pub fn node_from_query_result(mut result: QueryResult, shape: &str) -> Result<Node> {
    match shape {
        "val" | "row" => {
            let Some(mut row) = result.next() else {
                return Ok(Node::Null(Null));
            };

            if shape == "val" {
                if row.is_empty() {
                    return Ok(Node::Null(Null));
                }
                Ok(primitive_from_value(row.swap_remove(0)).into())
            } else {
                let values = row.into_iter().map(primitive_from_value).collect();
                Ok(Node::Array(Array(values)))
            }
        }

        "col" => {
            let values = result
                .map(|mut row| {
                    if row.is_empty() {
                        Primitive::Null(Null)
                    } else {
                        primitive_from_value(row.swap_remove(0))
                    }
                })
                .collect();
            Ok(Node::Array(Array(values)))
        }

        "all" => datatable_from_query_result(result).map(Node::Datatable),

        _ => bail!("Unknown shape for assigning query result: {shape}"),
    }
}

/// Create a Stencila [`Array`] of tuples of doc ids and node paths from a Kuzu [`QueryResult`]
pub fn excerpts_from_query_result(result: QueryResult) -> Result<Array> {
    let mut nodes = Vec::new();
    for row in result {
        for value in row {
            let Value::Node(node_val) = value else {
                bail!("Expected a Kuzu node");
            };

            let mut doc_id = None;
            let mut node_path = None;
            for (name, value) in node_val.get_properties() {
                if name == "docId" {
                    doc_id = Some(value.to_string());
                }

                if name == "nodePath" {
                    node_path = Some(value.to_string());
                }

                if doc_id.is_some() && node_path.is_some() {
                    break;
                }
            }
            let (Some(doc_id), Some(node_path)) = (doc_id, node_path) else {
                bail!("docId or nodePath fields missing")
            };

            nodes.push(Primitive::String([&doc_id, ":", &node_path].concat()))
        }
    }

    Ok(Array(nodes))
}

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
            label: String,
        },
        Edge {
            id: String,
            label: String,
            source: String,
            target: String,
        },
    }

    fn node(node: NodeVal) -> Element {
        let label = node.get_label_name().to_string();

        Element {
            data: Data::Node {
                id: node.get_node_id().to_string(),
                label,
            },
        }
    }

    fn edge(rel: RelVal) -> Element {
        let label = rel.get_label_name().to_string();
        let source = rel.get_src_node().to_string();
        let target = rel.get_dst_node().to_string();

        Element {
            data: Data::Edge {
                id: [&source, ".", &target].concat(),
                label,
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

    let mut style = vec![
        json!({
            "selector": "node",
            "style": {
                "label": "data(label)",
                "font-size": "12px",
            }
        }),
        json!({
            "selector": "edge",
            "style": {
                "curve-style": "bezier",
                "target-arrow-shape": "triangle",

                "label": "data(label)",
                "font-size": "10px",
                "color": "#666",
                "text-rotation": "autorotate",
                "text-margin-y": -10,
            }
        }),
    ];

    let uniques = elements
        .iter()
        .filter_map(|elem| match &elem.data {
            Data::Node { label, .. } => Some(label),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let count = uniques.len() as f32;
    for (index, label) in uniques.into_iter().enumerate() {
        let hue = (index as f32 / count) * 360.0;
        style.push(json!({
            "selector": format!("node[label = \"{label}\"]"),
            "style": {
                "background-color": format!("hsl({:.0}, 70%, 70%)", hue)
            }
        }));
    }

    let json = serde_json::to_string(&json!({
        "elements": elements,
        "style": style,
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
        LogicalType::List { child_type } => Some(Validator::ArrayValidator(ArrayValidator {
            items_validator: validator_from_logical_type(&child_type).map(Box::new),
            ..Default::default()
        })),
        LogicalType::Array {
            child_type,
            num_elements,
        } => Some(Validator::ArrayValidator(ArrayValidator {
            items_validator: validator_from_logical_type(&child_type).map(Box::new),
            min_items: Some(*num_elements as i64),
            max_items: Some(*num_elements as i64),
            ..Default::default()
        })),
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
        Value::List(.., value) => {
            let items = value.into_iter().map(primitive_from_value).collect();
            Primitive::Array(Array(items))
        }
        Value::Node(value) => {
            let mut props: IndexMap<String, Primitive> = value
                .get_properties()
                .iter()
                .map(|(name, value)| (name.clone(), primitive_from_value(value.clone())))
                .collect();
            props.insert(
                "_label".to_string(),
                Primitive::String(value.get_label_name().clone()),
            );
            Primitive::Object(Object(props))
        }
        Value::Rel(value) => {
            let mut props: IndexMap<String, Primitive> = value
                .get_properties()
                .iter()
                .map(|(name, value)| (name.clone(), primitive_from_value(value.clone())))
                .collect();
            props.insert(
                "_label".to_string(),
                Primitive::String(value.get_label_name().clone()),
            );
            Primitive::Object(Object(props))
        }
        _ => Primitive::String(value.to_string()),
    }
}

/// Create a Stencila [`ExecutionMessage`] from a Kuzu [`Error`]
pub fn execution_message_from_error(
    error: Error,
    query: &str,
    line_offset: usize,
) -> ExecutionMessage {
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
        match PARSER_EXC_REGEX.captures(rest) {
            Some(captures) => {
                let leading_lines = query
                    .chars()
                    .take_while(|&c| c == '\n')
                    .count()
                    .saturating_sub(1);

                code_location = Some(CodeLocation {
                    start_line: captures[1]
                        .parse()
                        .ok()
                        .map(|line: u64| line + line_offset as u64 + leading_lines as u64 - 1),
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
