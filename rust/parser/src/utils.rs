//! Utility functions for use by parser implementations

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use common::{
    once_cell::sync::Lazy,
    regex::{Captures, Regex},
};
use formats::Format;
use graph_triples::{
    relations::{self, NULL_RANGE},
    resources,
    stencila_schema::{CodeError, ExecutionAuto},
    Relation, Resource, Tag,
};

use crate::ResourceInfo;

/// Apply all comment tags in code where a comment is a line starting with the specified character
pub fn apply_tags_all(
    path: &Path,
    lang: Format,
    code: &str,
    chars: &str,
    kind: Option<String>,
    resource_info: &mut ResourceInfo,
) {
    for (row, line) in code.lines().enumerate() {
        if line.starts_with(chars) {
            apply_tags(path, lang, row, line, kind.clone(), resource_info);
        }
    }
}

/// Apply comment tags to a [`ResourceInfo`] object.
///
/// Parses tags from a comment and updates the supplied [`ResourceInfo`]. This pattern is
/// used because (a) the resource info may already be partially populated based on semantic
/// analysis of the code (which comments wish to override), and (b) there might be multiple
/// comments in a code resources each with (potentially overriding) tags
///
/// # Arguments
///
/// - `path`:    The path of the file.
///              Used, for example, when constructing `Symbol` resources for `@assigns` etc tags.
/// - `lang`:    The language of code that the comment is part of.
///              Used, for example, when constructing `Module` resources for `@imports` tags.
/// - `row`:     The line number of the start of the comment.
///              Used for constructing a `Range` for resources.
/// - `comment`: The comment from which to parse tags, usually a comment
/// - `kind`:    The default type for `Symbol` resources.
/// - `resource_info`: The [`ResourceInfo`] object to update
pub fn apply_tags(
    path: &Path,
    lang: Format,
    row: usize,
    comment: &str,
    kind: Option<String>,
    resource_info: &mut ResourceInfo,
) {
    let mut pairs: Vec<(Relation, Resource)> = Vec::new();
    let mut onlies: Vec<String> = Vec::new();
    for (index, line) in comment.lines().enumerate() {
        let range = (row + index, 0, row + index, line.len() - 1);
        if let Some(tag) = parse_tag(line) {
            // Record tags for potential use later when executed
            resource_info.tags.insert(tag.clone());

            let name = tag.name.as_str();
            let relation = match name {
                "pure" | "impure" => {
                    resource_info.execute_pure = Some(name == "pure");
                    continue;
                }

                "autorun" => {
                    let variant = match tag.value.as_str() {
                        "always" => Some(ExecutionAuto::Always),
                        "never" => Some(ExecutionAuto::Never),
                        _ => Some(ExecutionAuto::Needed),
                    };
                    resource_info.execute_auto = variant;
                    continue;
                }

                "imports" => relations::uses(range),
                "declares" => relations::declares(range),
                "assigns" => relations::assigns(range),
                "alters" => relations::alters(range),
                "uses" => relations::uses(range),
                "on" => relations::on(range),
                "reads" => relations::reads(range),
                "writes" => relations::writes(range),
                "requires" => relations::requires(range),

                _ => continue,
            };

            static REGEX_ITEMS: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"\s+|(\s*,\s*)").expect("Unable to create regex"));

            let args: Vec<String> = REGEX_ITEMS
                .split(&tag.value)
                .map(|item| item.to_string())
                .collect();

            for arg in args {
                if arg == "only" {
                    onlies.push(name.to_string());
                    continue;
                }

                let resource = match name {
                    "imports" => resources::module(lang, &arg),
                    "declares" | "assigns" | "alters" | "uses" | "on" => {
                        resources::symbol(path, &arg, &kind.clone().unwrap_or_default())
                    }
                    "reads" | "writes" => resources::file(&PathBuf::from(arg)),
                    "requires" => resources::code(path, &arg, "", Format::Unknown),
                    _ => continue,
                };
                pairs.push((relation.clone(), resource))
            }
        }
    }

    // Remove existing pairs for relation types where the `only` keyword is present in tags
    if let Some(relations) = &mut resource_info.relations {
        for only in onlies {
            relations.retain(|(relation, _resource)| {
                !(matches!(relation, Relation::Imports(..)) && only == "imports"
                    || matches!(relation, Relation::Declares(..)) && only == "declares"
                    || matches!(relation, Relation::Assigns(..)) && only == "assigns"
                    || matches!(relation, Relation::Alters(..)) && only == "alters"
                    || matches!(relation, Relation::Uses(..)) && only == "uses"
                    || matches!(relation, Relation::On(..)) && only == "on"
                    || matches!(relation, Relation::Reads(..)) && only == "reads"
                    || matches!(relation, Relation::Writes(..)) && only == "writes")
            })
        }
    }

    // Append pairs from tags
    if !pairs.is_empty() {
        if let Some(relations) = &mut resource_info.relations {
            relations.append(&mut pairs);
        } else {
            resource_info.relations = Some(pairs)
        }
    }
}

/// Parse a tag from a comment line
fn parse_tag(line: &str) -> Option<Tag> {
    static REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(@global\s+)?@([a-zA-Z]+)\s+(.*?)?\s*(:?\*/)?$")
            .expect("Unable to create regex")
    });

    REGEX.captures(line).map(|captures| {
        let global = captures.get(1).is_some();
        let name = captures[2].to_lowercase();
        let value = captures
            .get(3)
            .map_or_else(String::new, |group| group.as_str().to_string());
        Tag {
            name,
            value,
            global,
        }
    })
}

/// Regex for detecting variable interpolations within code
///
/// Allows for $var and ${var} patterns
pub static VAR_INTERP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\$([a-zA-Z][a-zA-Z_0-9]*)\b)|(?:\$\{\s*([a-zA-Z][a-zA-Z_0-9]*)\s*\})")
        .expect("Unable to create regex")
});

/// Parse variable interpolations in code into a vector of relations
///
/// Used by parsers to define the relations between some code and other resources.
pub fn parse_var_interps(code: &str, path: &Path) -> Vec<(Relation, Resource)> {
    VAR_INTERP_REGEX
        .captures_iter(code)
        .map(|captures| {
            let symbol = captures
                .get(1)
                .or_else(|| captures.get(2))
                .expect("Should always have one group");
            (
                relations::uses((0, symbol.start(), 0, symbol.end())),
                resources::symbol(path, symbol.as_str(), ""),
            )
        })
        .collect()
}

/// Perform variable interpolations in code using a map of symbols to strings
///
/// Used by kernels before executing code to perform interpolation. Returns
/// an error message for each variable that is not in the map.
pub fn perform_var_interps(
    code: &str,
    symbols: &HashMap<String, String>,
) -> (String, Vec<CodeError>) {
    let mut messages = Vec::new();
    let interpolated = VAR_INTERP_REGEX.replace_all(code, |captures: &Captures| {
        let symbol = captures
            .get(1)
            .or_else(|| captures.get(2))
            .expect("Should always have one group")
            .as_str();
        match symbols.get(symbol) {
            Some(value) => value.to_owned(),
            None => {
                messages.push(CodeError {
                    error_type: Some(Box::new("UnknownSymbol".to_string())),
                    error_message: format!("Symbol `{}` is not available", symbol),
                    ..Default::default()
                });
                captures[0].to_string()
            }
        }
    });
    (interpolated.to_string(), messages)
}

/// Regex for detecting file interpolations within code
///
/// Only allows for @{file} pattern
pub static FILE_INTERP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:@\{([^}]+)\})").expect("Unable to create regex"));

/// Parse file interpolations in code into a vector of relations
///
/// Used by parsers to define the relations between some code and other resources.
pub fn parse_file_interps(code: &str) -> Vec<(Relation, Resource)> {
    FILE_INTERP_REGEX
        .captures_iter(code)
        .map(|captures| {
            let file = captures.get(1).expect("Should always have one group");
            let path = PathBuf::from(file.as_str().trim());
            (relations::uses(NULL_RANGE), resources::file(&path))
        })
        .collect()
}

/// Perform file interpolations in code
pub fn perform_file_interps(code: &str, directory: &Path) -> (String, Vec<CodeError>) {
    let mut messages = Vec::new();
    let interpolated = FILE_INTERP_REGEX.replace_all(code, |captures: &Captures| {
        let file = captures
            .get(1)
            .expect("Should always have one group")
            .as_str();

        let file = PathBuf::from(file);
        let path = if file.is_relative() {
            directory.join(file)
        } else {
            file
        };

        if !path.exists() {
            messages.push(CodeError {
                error_type: Some(Box::new("PathError".to_string())),
                error_message: format!("File `{}` does not exist", path.display()),
                ..Default::default()
            });
            captures[0].to_string()
        } else {
            match std::fs::read_to_string(&path) {
                Ok(value) => value,
                Err(error) => {
                    messages.push(CodeError {
                        error_type: Some(Box::new("ReadError".to_string())),
                        error_message: format!(
                            "While interpolating file `{}`: {}",
                            path.display(),
                            error
                        ),
                        ..Default::default()
                    });
                    captures[0].to_string()
                }
            }
        }
    });
    (interpolated.to_string(), messages)
}

/// Is some text quoted?
pub fn is_quoted(text: &str) -> bool {
    (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
}

/// Remove single and double quotes from text
///
/// Useful for "unquoting" captured string literals.
pub fn remove_quotes(text: &str) -> String {
    if is_quoted(text) {
        let mut text = text.to_string();
        text.pop();
        text.remove(0);
        text
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_tag() {
        assert_eq!(
            parse_tag("-- @db sqlite://some/file.db3"),
            Some(Tag {
                name: "db".to_string(),
                value: "sqlite://some/file.db3".to_string(),
                global: false
            })
        );
        assert_eq!(
            parse_tag("-- @global @db postgres://user:pwd@host:5432/db"),
            Some(Tag {
                name: "db".to_string(),
                value: "postgres://user:pwd@host:5432/db".to_string(),
                global: true
            })
        );
    }
}
