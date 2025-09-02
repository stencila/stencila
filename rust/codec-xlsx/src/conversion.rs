use std::collections::HashMap;

use calamine::{Data, Range};

use stencila_codec::{
    eyre::Result,
    stencila_schema::{
        ArrayValidator, BooleanValidator, Datatable, DatatableColumn, IntegerValidator, Null,
        NumberValidator, Primitive, StringValidator, Validator,
    },
};

/// Convert a calamine [`Range`] to a Stencila [`Datatable`].
///
/// Extracts data from the Range, using the first row as column headers. Each subsequent
/// row becomes a data row in the Datatable. The function infers column types from the
/// data and creates appropriate validators for each column.
pub fn range_to_datatable(range: Range<Data>) -> Result<Datatable> {
    let (height, width) = range.get_size();

    if height == 0 || width == 0 {
        return Ok(Datatable::new(Vec::new()));
    }

    // Extract column headers from the first row
    let column_names: Vec<String> = (0..width)
        .map(|col| {
            range
                .get_value((0, col as u32))
                .map(data_to_string)
                .unwrap_or_else(|| format!("Column{}", col + 1))
        })
        .collect();

    // Initialize column data structures
    let mut column_data: Vec<Vec<Primitive>> = vec![Vec::new(); width];

    // Extract data rows (skip header row)
    for row in 1..height {
        for (col, column) in column_data.iter_mut().enumerate().take(width) {
            let cell_value = range
                .get_value((row as u32, col as u32))
                .map(data_to_primitive)
                .unwrap_or(Primitive::Null(Null));
            column.push(cell_value);
        }
    }

    // Create Datatable columns with inferred validators
    let columns: Vec<DatatableColumn> = column_names
        .into_iter()
        .zip(column_data)
        .map(|(column_name, values)| {
            let validator = infer_validator_from_data(&values);
            let mut column = DatatableColumn::new(column_name, values);
            column.validator = Some(validator);
            column
        })
        .collect();

    Ok(Datatable::new(columns))
}

/// Convert a calamine [`Data`] value to a Stencila [`Primitive`].
///
/// Maps calamine's Data enum variants to appropriate Stencila Primitive types,
/// handling type conversion and null values consistently across the conversion process.
fn data_to_primitive(data: &Data) -> Primitive {
    match data {
        Data::Empty => Primitive::Null(Null),
        Data::String(s) => Primitive::String(s.clone()),
        Data::Float(f) => {
            // Check if the float is actually an integer
            if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                Primitive::Integer(*f as i64)
            } else {
                Primitive::Number(*f)
            }
        }
        Data::Int(i) => Primitive::Integer(*i),
        Data::Bool(b) => Primitive::Boolean(*b),
        Data::DateTime(dt) => {
            // Convert Excel date to string representation
            // Note: calamine provides a datetime value, but we'll convert to string
            // for consistency with Stencila's schema handling
            Primitive::String(dt.to_string())
        }
        Data::DateTimeIso(dt) => Primitive::String(dt.clone()),
        Data::DurationIso(dur) => Primitive::String(dur.clone()),
        Data::Error(err) => Primitive::String(format!("ERROR: {err}")),
    }
}

/// Convert a calamine [`Data`] value to a string representation.
///
/// Provides consistent string conversion for column headers and other
/// display purposes, handling all Data variants appropriately.
fn data_to_string(data: &Data) -> String {
    match data {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(dt) => dt.clone(),
        Data::DurationIso(dur) => dur.clone(),
        Data::Error(err) => format!("ERROR: {err}"),
    }
}

/// Infer the appropriate [`ArrayValidator`] from a collection of [`Primitive`] values.
///
/// Analyzes the data types in a column to determine the most appropriate validator,
/// following a precedence order that prefers more specific types over generic ones.
/// This ensures optimal type safety while handling mixed-type columns gracefully.
fn infer_validator_from_data(values: &[Primitive]) -> ArrayValidator {
    let mut type_counts = HashMap::new();

    // Count occurrences of each type (excluding nulls)
    for value in values {
        let type_name = match value {
            Primitive::Null(_) => continue, // Skip nulls for type inference
            Primitive::Boolean(_) => "boolean",
            Primitive::Integer(_) | Primitive::UnsignedInteger(_) => "integer",
            Primitive::Number(_) => "number",
            Primitive::String(_) | Primitive::Array(_) | Primitive::Object(_) => "string",
        };
        *type_counts.entry(type_name).or_insert(0) += 1;
    }

    // Determine the most appropriate validator based on type prevalence
    let items_validator = match type_counts.len() {
        0 => {
            // All values are null, default to string
            Some(Box::new(Validator::StringValidator(StringValidator::new())))
        }
        1 => {
            // All non-null values are the same type
            let (type_name, _) = type_counts
                .iter()
                .next()
                .expect("type_counts should not be empty");
            Some(Box::new(match *type_name {
                "boolean" => Validator::BooleanValidator(BooleanValidator::new()),
                "integer" => Validator::IntegerValidator(IntegerValidator::new()),
                "number" => Validator::NumberValidator(NumberValidator::new()),
                _ => Validator::StringValidator(StringValidator::new()),
            }))
        }
        _ => {
            // Mixed types - prefer numeric if we have numbers/integers, otherwise string
            Some(Box::new(if type_counts.contains_key("number") {
                Validator::NumberValidator(NumberValidator::new())
            } else if type_counts.contains_key("integer") {
                Validator::IntegerValidator(IntegerValidator::new())
            } else {
                // Mixed non-numeric types, fall back to string
                Validator::StringValidator(StringValidator::new())
            }))
        }
    };

    let mut validator = ArrayValidator::new();
    validator.items_validator = items_validator;
    validator
}
