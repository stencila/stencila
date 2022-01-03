//! Utility functions for use by parser implementations

use graph_triples::{relations, resources, Relation, Resource};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

use crate::ParseInfo;

/// Apply comment tags to a [`ParseInfo`] object.
///
/// Parses tags from a comment and updates the supplied [`ParseInfo`]. This pattern is
/// used because (a) the parse info may already be partially populated based on semantic
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
pub fn apply_tags(
    path: &Path,
    lang: &str,
    row: usize,
    comment: &str,
    kind: Option<String>,
    parse_info: &mut ParseInfo,
) {
    static REGEX_TAGS: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"@(imports|assigns|alters|uses|reads|writes|pure|impure)\s+(.*?)(\*/)?$")
            .expect("Unable to create regex")
    });
    static REGEX_ITEMS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\s+|(\s*,\s*)").expect("Unable to create regex"));

    let mut pairs: Vec<(Relation, Resource)> = Vec::new();
    let mut onlies: Vec<String> = Vec::new();
    for (index, line) in comment.lines().enumerate() {
        let range = (row + index, 0, row + index, line.len() - 1);
        if let Some(captures) = REGEX_TAGS.captures(line) {
            let tag = captures[1].to_string();

            let relation = match tag.as_str() {
                "pure" => {
                    parse_info.pure = Some(true);
                    continue;
                }
                "impure" => {
                    parse_info.pure = Some(false);
                    continue;
                }
                "imports" => relations::uses(range),
                "assigns" => relations::assigns(range),
                "alters" => relations::alters(range),
                "uses" => relations::uses(range),
                "reads" => relations::reads(range),
                "writes" => relations::writes(range),
                _ => continue,
            };

            let items: Vec<String> = REGEX_ITEMS
                .split(captures[2].trim())
                .map(|item| item.to_string())
                .collect();

            for item in items {
                if item == "only" {
                    onlies.push(tag.clone());
                    continue;
                }

                let resource = match tag.as_str() {
                    "imports" => resources::module(lang, &item),
                    "assigns" | "alters" | "uses" => {
                        resources::symbol(path, &item, &kind.clone().unwrap_or_default())
                    }
                    "reads" | "writes" => resources::file(&PathBuf::from(item)),
                    _ => continue,
                };
                pairs.push((relation.clone(), resource))
            }
        }
    }

    // Remove existing pairs for relation types where the `only` keyword is present in tags
    for only in onlies {
        parse_info.relations.retain(|(relation, _resource)| {
            !(matches!(relation, Relation::Import(..)) && only == "imports"
                || matches!(relation, Relation::Assign(..)) && only == "assigns"
                || matches!(relation, Relation::Alter(..)) && only == "alters"
                || matches!(relation, Relation::Use(..)) && only == "uses"
                || matches!(relation, Relation::Read(..)) && only == "reads"
                || matches!(relation, Relation::Write(..)) && only == "writes")
        })
    }

    // Append pairs from tags
    parse_info.relations.append(&mut pairs);
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
