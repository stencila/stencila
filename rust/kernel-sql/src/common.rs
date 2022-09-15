use kernel::{
    common::itertools::Itertools,
    stencila_schema::{Datatable, ValidatorTypes},
};

/// Generate a `CREATE TABLE` statement for a Stencila [`Datatable`]
pub(crate) fn create_table_from_datatable(
    name: &str,
    datatable: &Datatable,
    or_replace: bool,
) -> String {
    let columns = datatable
        .columns
        .iter()
        .map(|column| {
            let item_validator = column
                .validator
                .as_deref()
                .and_then(|array_validator| array_validator.items_validator.clone());
            let data_type = match item_validator.as_deref() {
                Some(ValidatorTypes::BooleanValidator(..)) => "BOOLEAN",
                Some(ValidatorTypes::IntegerValidator(..)) => "INTEGER",
                Some(ValidatorTypes::NumberValidator(..)) => "REAL",
                Some(ValidatorTypes::StringValidator(..)) => "TEXT",
                Some(ValidatorTypes::DateValidator(..)) => "DATE",
                Some(ValidatorTypes::TimeValidator(..)) => "TIME",
                Some(ValidatorTypes::DateTimeValidator(..)) => "TIMESTAMP",
                Some(ValidatorTypes::TimestampValidator(..)) => "TIMESTAMP",
                Some(ValidatorTypes::DurationValidator(..)) => "INTERVAL",
                _ => "JSON",
            };

            let is_nullable = if let Some(validator) = &column.validator {
                if !validator.items_nullable {
                    "NOT NULL".to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            format!("{} {} {}", column.name, data_type, is_nullable)
        })
        .collect_vec()
        .join(", ");

    format!(
        "CREATE {} TABLE \"{}\"({});\n",
        if or_replace { "OR REPLACE" } else { "" },
        name,
        columns,
    )
}

/// Get the number of rows and columns of a Stencila [`Datatable`]
pub(crate) fn datatable_rows_cols(datatable: &Datatable) -> (usize, usize) {
    let rows = datatable
        .columns
        .first()
        .map(|column| column.values.len())
        .unwrap_or(0);

    let cols = datatable.columns.len();

    (rows, cols)
}
