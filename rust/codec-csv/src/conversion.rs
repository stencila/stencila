use codec::{
    common::eyre::Result,
    schema::{
        ArrayValidator, BooleanValidator, Datatable, DatatableColumn, DateTimeValidator,
        DateValidator, IntegerValidator, Null, NumberValidator, Primitive, StringValidator,
        Validator,
    },
};
use polars::prelude::*;

/// Convert a Polars [`DataFrame`] to a Stencila [`Datatable`].
///
/// Transforms each DataFrame column into a [`DatatableColumn`] with appropriate
/// type validators. This conversion preserves the data structure while adding
/// Stencila's schema validation capabilities to each column.
pub fn dataframe_to_datatable(df: DataFrame) -> Result<Datatable> {
    let mut columns = Vec::new();

    for series in df.iter() {
        let name = series.name().to_string();
        let validator = dtype_to_validator(series.dtype());
        let values = series_to_primitives(series)?;

        let mut column = DatatableColumn::new(name, values);
        column.validator = Some(validator);
        columns.push(column);
    }

    Ok(Datatable::new(columns))
}

/// Convert a Stencila [`Datatable`] to a Polars [`DataFrame`].
///
/// Transforms each [`DatatableColumn`] into a Polars Series, using the column's
/// validator to determine the appropriate data type. This enables efficient
/// processing and serialization using Polars' optimized operations.
pub fn datatable_to_dataframe(datatable: &Datatable) -> Result<DataFrame> {
    let mut series_vec = Vec::new();

    for column in &datatable.columns {
        let series = primitives_to_series(&column.name, &column.values, &column.validator)?;
        series_vec.push(series.into());
    }

    Ok(DataFrame::new(series_vec)?)
}

/// Create an [`ArrayValidator`] from a Polars [`DataType`].
///
/// Maps Polars data types to appropriate Stencila validators for schema validation.
/// This ensures that data type constraints are preserved when converting from
/// DataFrame columns to [`DatatableColumn`] structures.
pub fn dtype_to_validator(dtype: &DataType) -> ArrayValidator {
    let items_validator = match dtype {
        DataType::Boolean => Some(Box::new(Validator::BooleanValidator(
            BooleanValidator::new(),
        ))),
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => Some(Box::new(
            Validator::IntegerValidator(IntegerValidator::new()),
        )),
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
            let mut validator = IntegerValidator::new();
            validator.minimum = Some(0.0);
            Some(Box::new(Validator::IntegerValidator(validator)))
        }
        DataType::Float32 | DataType::Float64 => {
            Some(Box::new(Validator::NumberValidator(NumberValidator::new())))
        }
        DataType::String => Some(Box::new(Validator::StringValidator(StringValidator::new()))),
        DataType::Date => Some(Box::new(Validator::DateValidator(DateValidator::new()))),
        DataType::Datetime(_, _) => Some(Box::new(Validator::DateTimeValidator(
            DateTimeValidator::new(),
        ))),
        _ => None,
    };

    let mut validator = ArrayValidator::new();
    validator.items_validator = items_validator;
    validator
}

/// Convert a Polars [`Series`] to a vector of Stencila [`Primitive`] values.
///
/// Handles type-specific conversion for all supported Polars data types, including
/// proper null value handling. Unsupported types are converted to string representations
/// to ensure data preservation during the conversion process.
pub fn series_to_primitives(series: &Series) -> Result<Vec<Primitive>> {
    let mut values = Vec::with_capacity(series.len());

    match series.dtype() {
        DataType::Boolean => {
            let ca = series.bool()?;
            for opt_val in ca.iter() {
                values.push(match opt_val {
                    Some(v) => Primitive::Boolean(v),
                    None => Primitive::Null(Null),
                });
            }
        }
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => {
            let ca = series.i64()?;
            for opt_val in ca.iter() {
                values.push(match opt_val {
                    Some(v) => Primitive::Integer(v),
                    None => Primitive::Null(Null),
                });
            }
        }
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
            let ca = series.u64()?;
            for opt_val in ca.iter() {
                values.push(match opt_val {
                    Some(v) => Primitive::UnsignedInteger(v),
                    None => Primitive::Null(Null),
                });
            }
        }
        DataType::Float32 | DataType::Float64 => {
            let ca = series.f64()?;
            for opt_val in ca.iter() {
                values.push(match opt_val {
                    Some(v) => Primitive::Number(v),
                    None => Primitive::Null(Null),
                });
            }
        }
        DataType::String => {
            let ca = series.str()?;
            for opt_val in ca.iter() {
                values.push(match opt_val {
                    Some(v) => Primitive::String(v.to_string()),
                    None => Primitive::Null(Null),
                });
            }
        }
        _ => {
            // For other types, convert to string representation
            for i in 0..series.len() {
                let val = series.get(i)?;
                values.push(Primitive::String(format!("{val}")));
            }
        }
    }

    Ok(values)
}

/// Convert a vector of Stencila [`Primitive`] values to a Polars [`Series`].
///
/// Uses the optional [`ArrayValidator`] to determine the target data type, falling back
/// to type inference from the first non-null value. This ensures optimal storage
/// efficiency while preserving the original data types from the [`Datatable`].
pub fn primitives_to_series(
    name: &str,
    values: &[Primitive],
    validator: &Option<ArrayValidator>,
) -> Result<Series> {
    // Try to infer the type from the validator if present
    if let Some(array_validator) = validator {
        if let Some(items_validator) = &array_validator.items_validator {
            return match &**items_validator {
                Validator::BooleanValidator(_) => {
                    let vec: Vec<Option<bool>> = values
                        .iter()
                        .map(|p| match p {
                            Primitive::Boolean(b) => Some(*b),
                            Primitive::Null(_) => None,
                            _ => None,
                        })
                        .collect();
                    Ok(Series::new(name.into(), vec))
                }
                Validator::IntegerValidator(_) => {
                    let vec: Vec<Option<i64>> = values
                        .iter()
                        .map(|p| match p {
                            Primitive::Integer(i) => Some(*i),
                            Primitive::UnsignedInteger(u) => Some(*u as i64),
                            Primitive::Null(_) => None,
                            _ => None,
                        })
                        .collect();
                    Ok(Series::new(name.into(), vec))
                }
                Validator::NumberValidator(_) => {
                    let vec: Vec<Option<f64>> = values
                        .iter()
                        .map(|p| match p {
                            Primitive::Number(n) => Some(*n),
                            Primitive::Integer(i) => Some(*i as f64),
                            Primitive::UnsignedInteger(u) => Some(*u as f64),
                            Primitive::Null(_) => None,
                            _ => None,
                        })
                        .collect();
                    Ok(Series::new(name.into(), vec))
                }
                _ => {
                    // Fall back to string for other types
                    let vec: Vec<Option<String>> = values
                        .iter()
                        .map(|p| match p {
                            Primitive::String(s) => Some(s.clone()),
                            Primitive::Null(_) => None,
                            _ => Some(format!("{p:?}")),
                        })
                        .collect();
                    Ok(Series::new(name.into(), vec))
                }
            };
        }
    }

    // If no validator, try to infer from the data
    // Check the first non-null value to determine the type
    let first_non_null = values.iter().find(|p| !matches!(p, Primitive::Null(_)));

    match first_non_null {
        Some(Primitive::Boolean(_)) => {
            let vec: Vec<Option<bool>> = values
                .iter()
                .map(|p| match p {
                    Primitive::Boolean(b) => Some(*b),
                    Primitive::Null(_) => None,
                    _ => None,
                })
                .collect();
            Ok(Series::new(name.into(), vec))
        }
        Some(Primitive::Integer(_)) | Some(Primitive::UnsignedInteger(_)) => {
            let vec: Vec<Option<i64>> = values
                .iter()
                .map(|p| match p {
                    Primitive::Integer(i) => Some(*i),
                    Primitive::UnsignedInteger(u) => Some(*u as i64),
                    Primitive::Null(_) => None,
                    _ => None,
                })
                .collect();
            Ok(Series::new(name.into(), vec))
        }
        Some(Primitive::Number(_)) => {
            let vec: Vec<Option<f64>> = values
                .iter()
                .map(|p| match p {
                    Primitive::Number(n) => Some(*n),
                    Primitive::Integer(i) => Some(*i as f64),
                    Primitive::UnsignedInteger(u) => Some(*u as f64),
                    Primitive::Null(_) => None,
                    _ => None,
                })
                .collect();
            Ok(Series::new(name.into(), vec))
        }
        _ => {
            // Default to string
            let vec: Vec<Option<String>> = values
                .iter()
                .map(|p| match p {
                    Primitive::String(s) => Some(s.clone()),
                    Primitive::Null(_) => None,
                    _ => Some(format!("{p:?}")),
                })
                .collect();
            Ok(Series::new(name.into(), vec))
        }
    }
}
