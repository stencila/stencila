use kuzu::{LogicalType, QueryResult, Value};

use kernel::{
    common::eyre::{Result, bail},
    schema::*,
};

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
