//! Utility functions associated with compiling code

use graphs::{relations, resources, Relation, Resource};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Whether or not the text is quoted
pub(crate) fn is_quoted(text: &str) -> bool {
    (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
}

/// Remove single and double quotes from text
///
/// Useful for "unquoting" captured string literals.
pub(crate) fn remove_quotes(text: &str) -> String {
    text.replace(&['\"', '\''][..], "")
}

/// Parse a comment into a set of `Relation`/`Resource` pairs and the name relation
/// types for which those specified should be the only relations
pub(crate) fn parse_tags(
    path: &Path,
    lang: &str,
    row: usize,
    comment: &str,
    kind: Option<String>,
) -> (Vec<(Relation, Resource)>, Vec<String>) {
    static REGEX_TAG: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"@(imports|assigns|uses|modifies|reads|writes)\s+(.*?)(\*/)?$")
            .expect("Unable to create regex")
    });
    static REGEX_ITEMS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\s+|(\s*,\s*)").expect("Unable to create regex"));

    let kind = kind.unwrap_or_else(|| "".to_string());

    let mut relations: Vec<(Relation, Resource)> = Vec::new();
    let mut only: Vec<String> = Vec::new();
    for (index, line) in comment.lines().enumerate() {
        let range = (row + index, 0, row + index, line.len() - 1);
        if let Some(captures) = REGEX_TAG.captures(line) {
            let tag = captures[1].to_string();
            let relation = match tag.as_str() {
                "imports" => relations::uses(range),
                "assigns" => relations::assigns(range),
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
                    "assigns" | "uses" => resources::symbol(path, &item, &kind),
                    "reads" | "writes" => resources::file(&PathBuf::from(item)),
                    _ => continue,
                };
                relations.push((relation.clone(), resource))
            }
        }
    }
    (relations, only)
}
