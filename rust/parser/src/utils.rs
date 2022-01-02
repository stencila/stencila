//! Utility functions for use by parser implementations

use graph_triples::{relations, resources, Pairs, Relation, Resource};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Parse tags in a comment into a set of [`Relation`]-[`Resource`] pairs (and the names of relations
/// for which those declared should be the only relations included).
///
/// # Arguments
///
/// - `path`: The path of the file.
///           Used, for example, when constructing `Symbol` resources for `@assigns` etc tags.
/// - `lang`: The language of code that the comment is part of.
///           Used, for example, when constructing `Module` resources for `@imports` tags.
/// - `row`:  The line number of the start of the comment.
///           Used for constructing a `Range` for resources.
/// - `comment`: The comment from which to parse tags, usually a comment
/// - `kind`: The default type for `Symbol` resources.
pub fn parse_tags(
    path: &Path,
    lang: &str,
    row: usize,
    comment: &str,
    kind: Option<String>,
) -> (Vec<(Relation, Resource)>, Vec<String>) {
    static REGEX_TAG: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"@(imports|assigns|alters|uses|modifies|reads|writes)\s+(.*?)(\*/)?$")
            .expect("Unable to create regex")
    });
    static REGEX_ITEMS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\s+|(\s*,\s*)").expect("Unable to create regex"));

    let kind = kind.unwrap_or_else(|| "".to_string());

    let mut pairs: Vec<(Relation, Resource)> = Vec::new();
    let mut only: Vec<String> = Vec::new();
    for (index, line) in comment.lines().enumerate() {
        let range = (row + index, 0, row + index, line.len() - 1);
        if let Some(captures) = REGEX_TAG.captures(line) {
            let tag = captures[1].to_string();
            let relation = match tag.as_str() {
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
                    only.push(tag.clone());
                    continue;
                }

                let resource = match tag.as_str() {
                    "imports" => resources::module(lang, &item),
                    "assigns" | "alters" | "uses" => resources::symbol(path, &item, &kind),
                    "reads" | "writes" => resources::file(&PathBuf::from(item)),
                    _ => continue,
                };
                pairs.push((relation.clone(), resource))
            }
        }
    }
    (pairs, only)
}

/// Apply the [`Relation`]-[`Resource`] pairs declared in a comment to an existing set of pairs.
///
/// See [`parse_tags`] for details on arguments.
pub fn apply_tags(
    path: &Path,
    lang: &str,
    row: usize,
    comment: &str,
    kind: Option<String>,
    pairs: &mut Pairs,
) {
    // Parse tags into relations
    let (mut declared_pairs, only_relations) = parse_tags(path, lang, row, comment, kind);

    // Remove existing relations for relation types where the `only` keyword is present
    for only in only_relations {
        pairs.retain(|(relation, _resource)| {
            !(matches!(relation, Relation::Import(..)) && only == "imports"
                || matches!(relation, Relation::Assign(..)) && only == "assigns"
                || matches!(relation, Relation::Alter(..)) && only == "alters"
                || matches!(relation, Relation::Use(..)) && only == "uses"
                || matches!(relation, Relation::Read(..)) && only == "reads"
                || matches!(relation, Relation::Write(..)) && only == "writes")
        })
    }

    // Append declared pairs
    pairs.append(&mut declared_pairs);
}

/// Is som text quoted?
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
