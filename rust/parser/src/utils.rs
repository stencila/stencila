//! Utility functions for use by parser implementations

use std::path::{Path, PathBuf};

use common::{once_cell::sync::Lazy, regex::Regex};
use graph_triples::{
    relations, resources, stencila_schema::CodeChunkExecuteAuto, Relation, Resource, Tag,
};

use crate::ResourceInfo;

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
    lang: &str,
    row: usize,
    comment: &str,
    kind: Option<String>,
    resource_info: &mut ResourceInfo,
) {
    let mut pairs: Vec<(Relation, Resource)> = Vec::new();
    let mut onlies: Vec<String> = Vec::new();
    for (index, line) in comment.lines().enumerate() {
        let range = (row + index, 0, row + index, line.len() - 1);
        if let Some(tag) = parse_comment_line(line) {
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
                        "always" => Some(CodeChunkExecuteAuto::Always),
                        "never" => Some(CodeChunkExecuteAuto::Never),
                        _ => Some(CodeChunkExecuteAuto::Needed),
                    };
                    resource_info.execute_auto = variant;
                    continue;
                }

                "imports" => relations::uses(range),
                "declares" => relations::declares(range),
                "assigns" => relations::assigns(range),
                "alters" => relations::alters(range),
                "uses" => relations::uses(range),
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
                    "declares" | "assigns" | "alters" | "uses" => {
                        resources::symbol(path, &arg, &kind.clone().unwrap_or_default())
                    }
                    "reads" | "writes" => resources::file(&PathBuf::from(arg)),
                    "requires" => resources::code(path, &arg, "", None),
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
fn parse_comment_line(line: &str) -> Option<Tag> {
    static REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(global)?\s*@([a-zA-Z]+)\s+(.*?)?\s*(:?\*/)?$")
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

/// Is some text quoted?
pub fn is_quoted(text: &str) -> bool {
    (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
}

/// Remove single and double quotes from text
///
/// Useful for "unquoting" captured string literals.
pub fn remove_quotes(text: &str) -> String {
    text.replace(&['\"', '\''][..], "")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_comment_line() {
        assert_eq!(
            parse_comment_line("-- @db sqlite://some/file.db3"),
            Some(Tag {
                name: "db".to_string(),
                value: "sqlite://some/file.db3".to_string(),
                global: false
            })
        );
        assert_eq!(
            parse_comment_line("-- global @db postgres://user:pwd@host:5432/db"),
            Some(Tag {
                name: "db".to_string(),
                value: "postgres://user:pwd@host:5432/db".to_string(),
                global: true
            })
        );
    }
}
