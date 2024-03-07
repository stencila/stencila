//! Functions used as filters and elsewhere in `minijinja` templates

use assistant::{
    common::eyre::{eyre, Report},
    schema::{ArrayHint, Hint, Variable},
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
        Hint::ArrayHint(ArrayHint {
            length,
            item_types,
            minimum,
            maximum,
            nulls,
            ..
        }) => {
            desc += &format!("\n    - with length {length}");

            if let Some(item_types) = item_types {
                desc += &format!(
                    "\n    - containing values of the following types: {}",
                    item_types.join(", ")
                );
            }

            if let Some(minimum) = minimum {
                desc += &format!("\n    - with a minimum value of: {minimum}");
            }

            if let Some(maximum) = maximum {
                desc += &format!("\n    - with a maximum value of: {maximum}");
            }

            if let Some(nulls) = nulls {
                desc += &format!("\n    - containing {nulls} null values");
            }
        }
        _ => {
            // TODO handle all the other hint types
        }
    }

    desc
}
