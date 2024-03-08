//! Functions used as filters and elsewhere in `minijinja` templates

use assistant::{
    common::eyre::{eyre, Report},
    schema::{ArrayHint, DatatableColumnHint, DatatableHint, Hint, Variable},
};
use minijinja::{value::ViaDeserialize, Error};

/// Expand a `minijinja` error to include the sources of the error (location etc)
pub fn minijinja_error_to_eyre(error: Error) -> Report {
    let mut error = &error as &dyn std::error::Error;
    let mut message = format!("{error:#}");
    while let Some(source) = error.source() {
        message.push_str(&format!("\n{:#}", source));
        error = source;
    }
    eyre!(message)
}

/// Trim the starting characters from a string so that it is no longer than `length`
pub fn trim_start_chars(content: &str, length: u32) -> String {
    let current_length = content.chars().count();
    content
        .chars()
        .skip(current_length.saturating_sub(length as usize))
        .take(length as usize)
        .collect()
}

/// Trim the ending characters from a string so that it is no longer than `length`
pub fn trim_end_chars(content: &str, length: u32) -> String {
    content.chars().take(length as usize).collect()
}

fn describe_array_hint(hint: &ArrayHint) -> String {
    let mut lines = vec![format!(" with length {}", hint.length)];
    if let Some(item_types) = &hint.item_types {
        lines.push(format!(
            "containing values of the following types: {}",
            item_types.join(",")
        ));
    }
    if let Some(minimum) = &hint.minimum {
        lines.push(format!("with a minimum value of: {minimum}"));
    }
    if let Some(maximum) = &hint.maximum {
        lines.push(format!("with a maximum value of: {maximum}"));
    }
    if let Some(nulls) = &hint.nulls {
        lines.push(format!("containing {nulls} null values"));
    }
    lines.join("\n    - ")
}

fn describe_datatable_hint(hint: &DatatableHint) -> String {
    let mut header = format!(" with {} rows", hint.rows);
    if hint.columns.is_empty() {
        return header;
    }
    header += &format!(", with these columns:");
    let mut lines = vec![header];
    for column in &hint.columns {
        lines.push(format!("`{}`: type {}", column.name, column.item_type));
    }
    lines.join("\n    - ")
}

/// Create an Markdown description of a `Variable` as a list item with a
/// nested child list describing its characteristics.
pub fn describe_variable(variable: ViaDeserialize<Variable>) -> String {
    let mut desc = format!("- Variable `{}`", variable.name);

    if let Some(native_type) = &variable.native_type {
        desc += &format!(" is a `{native_type}`");
    }

    let Some(hint) = &variable.hint else {
        return desc;
    };

    match hint {
        Hint::ArrayHint(hint) => desc += &describe_array_hint(hint),
        Hint::DatatableHint(hint) => desc += &describe_datatable_hint(hint),
        _ => {
            // TODO handle all the other hint types
        }
    }

    desc
}
