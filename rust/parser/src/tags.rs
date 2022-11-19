use std::path::{Path, PathBuf};

use common::{itertools::Itertools, once_cell::sync::Lazy, regex::Regex, serde::Serialize};
use stencila_schema::{
    ExecutionAuto, ExecutionDependency, ExecutionDependencyNode, ExecutionDependencyRelation,
    ExecutionDependent, ExecutionDependentNode, ExecutionDependentRelation, ExecutionTag, File,
    SoftwareSourceCode, Variable,
};

use crate::ParseInfo;

/// A collection of tags
///
/// Implements a `HashMap` like interface but is implemented as a `Vec` as this
/// is expected to be more performant (in memory and CPU) given that the number
/// of tags in a `TagMap` will usually be small (<10).
#[derive(Debug, Default, Clone, Serialize)]
#[serde(transparent, crate = "common::serde")]
pub struct TagMap {
    inner: Vec<ExecutionTag>,
}

impl TagMap {
    /// Create a new tag map from a list of name/value pairs
    pub fn from_name_values(pairs: &[(&str, &str)]) -> Self {
        let mut map = Self::default();
        for (name, value) in pairs {
            map.insert(ExecutionTag {
                name: name.to_string(),
                value: value.to_string(),
                ..Default::default()
            });
        }
        map
    }

    /// Get a tag by name
    pub fn get(&self, name: &str) -> Option<&ExecutionTag> {
        self.inner.iter().find(|tag| tag.name == name)
    }

    /// Get a tag value by name
    pub fn get_value(&self, name: &str) -> Option<String> {
        self.get(name).map(|tag| tag.value.clone())
    }

    /// Get a tag split into individual space or comma separated items
    pub fn get_items(&self, name: &str) -> Vec<String> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\s+|(\s*,\s*)").expect("Unable to create regex"));

        match self.get_value(name) {
            Some(value) => REGEX.split(&value).map(String::from).collect_vec(),
            None => Vec::new(),
        }
    }

    /// Insert a tag
    ///
    /// Overrides any existing tag with the same `name`.
    pub fn insert(&mut self, new: ExecutionTag) {
        if let Some((position, ..)) = self.inner.iter().find_position(|tag| tag.name == new.name) {
            self.inner[position] = new;
        } else {
            self.inner.push(new)
        }
    }

    /// Insert `global` tags from another tag map
    ///
    /// Used to merge a resource's global tags into a document's global tags.
    pub fn insert_globals(&mut self, other: &TagMap) {
        for tag in other.inner.iter() {
            if tag.is_global {
                self.insert(tag.clone());
            }
        }
    }

    /// Merge tags from one tag map into another, overriding any duplicates
    ///
    /// Used to merge document's global tags into a resource's tags.
    pub fn merge(&self, other: &TagMap) -> TagMap {
        let mut clone = self.clone();
        for tag in &other.inner {
            clone.insert(tag.clone());
        }
        clone
    }
}

/// Apply all comment tags in code where a comment is a line starting with the specified character
pub fn apply_all_comment_tags(
    parse_info: &mut ParseInfo,
    code: &str,
    path: Option<&Path>,
    chars: &str,
) {
    for (row, line) in code.lines().enumerate() {
        if line.starts_with(chars) {
            apply_comment_tags(parse_info, line, path, row);
        }
    }
}

/// Apply comment tags to a [`ParseInfo`] object.
///
/// Parses tags from a comment and updates the supplied [`ParseInfo`]. This pattern is
/// used because (a) the resource info may already be partially populated based on semantic
/// analysis of the code (which comments wish to override), and (b) there might be multiple
/// comments in a code resources each with (potentially overriding) tags
///
/// # Arguments
///
/// - `parse_info`: The [`ParseInfo`] object to update
/// - `comment`: The comment from which to parse tags
/// - `path`:    The path of the file.
///              Used, for example, when constructing `File` resources for `@reads` tags.
/// - `row`:     The line number of the start of the comment. Used for constructing a `codeLocation`.
pub fn apply_comment_tags(
    parse_info: &mut ParseInfo,
    comment: &str,
    path: Option<&Path>,
    row: usize,
) {
    let mut dependencies: Vec<ExecutionDependency> = Vec::new();
    let mut dependents: Vec<ExecutionDependent> = Vec::new();
    let mut onlies: Vec<String> = Vec::new();
    for (index, line) in comment.lines().enumerate() {
        if let Some(tag) = parse_tag(line) {
            // Record all tags for potential use later when executed
            parse_info.execution_tags.push(tag.clone());

            let name = tag.name.as_str();

            if name == "pure" || name == "impure" {
                parse_info.execution_pure = Some(name == "pure");
                continue;
            }

            if name == "autorun" {
                let variant = match tag.value.as_str() {
                    "always" => Some(ExecutionAuto::Always),
                    "never" => Some(ExecutionAuto::Never),
                    _ => Some(ExecutionAuto::Needed),
                };
                parse_info.execution_auto = variant;
                continue;
            }

            let (dependency_relation, dependent_relation) = match name {
                "imports" => (Some(ExecutionDependencyRelation::Imports), None),
                "reads" => (Some(ExecutionDependencyRelation::Reads), None),
                "uses" => (Some(ExecutionDependencyRelation::Uses), None),

                "assigns" => (None, Some(ExecutionDependentRelation::Assigns)),
                "alters" => (None, Some(ExecutionDependentRelation::Alters)),
                "declares" => (None, Some(ExecutionDependentRelation::Declares)),
                "writes" => (None, Some(ExecutionDependentRelation::Writes)),

                _ => continue,
            };

            static REGEX_ITEMS: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"\s+|(\s*,\s*)").expect("Unable to create regex"));

            let code_location = Some([row + index, 0, row + index, line.len() - 1]);

            let args: Vec<String> = REGEX_ITEMS
                .split(&tag.value)
                .map(|item| item.to_string())
                .collect();

            for arg in args {
                if arg == "only" {
                    onlies.push(name.to_string());
                    continue;
                }

                if let Some(dependency_relation) = dependency_relation.clone() {
                    let dependency_node = match name {
                        "imports" => {
                            ExecutionDependencyNode::SoftwareSourceCode(SoftwareSourceCode {
                                name: Some(Box::new(arg)),
                                ..Default::default()
                            })
                        }
                        "reads" => ExecutionDependencyNode::File(File {
                            path: path
                                .map_or_else(
                                    || PathBuf::from(arg.as_str()),
                                    |path| path.join(arg.as_str()),
                                )
                                .to_string_lossy()
                                .to_string(),
                            ..Default::default()
                        }),
                        "uses" => ExecutionDependencyNode::Variable(Variable {
                            namespace: path
                                .map(|path| Box::new(path.to_string_lossy().to_string())),
                            name: arg,
                            ..Default::default()
                        }),
                        _ => continue,
                    };
                    dependencies.push(ExecutionDependency {
                        dependency_relation,
                        dependency_node,
                        code_location,
                        ..Default::default()
                    })
                } else if let Some(dependent_relation) = dependent_relation.clone() {
                    let dependent_node = match name {
                        "assigns" | "alters" | "declares" => {
                            ExecutionDependentNode::Variable(Variable {
                                namespace: path
                                    .map(|path| Box::new(path.to_string_lossy().to_string())),
                                name: arg,
                                ..Default::default()
                            })
                        }
                        "write" => ExecutionDependentNode::File(File {
                            path: path
                                .map_or_else(
                                    || PathBuf::from(arg.as_str()),
                                    |path| path.join(arg.as_str()),
                                )
                                .to_string_lossy()
                                .to_string(),
                            ..Default::default()
                        }),
                        _ => continue,
                    };
                    dependents.push(ExecutionDependent {
                        dependent_relation,
                        dependent_node,
                        code_location,
                        ..Default::default()
                    })
                }
            }
        }
    }

    // Remove existing pairs from parse_info for tags where the `only` keyword is present in tags
    for only in onlies {
        parse_info.execution_dependencies.retain(
            |ExecutionDependency {
                 dependency_relation: relation,
                 ..
             }| {
                !(matches!(relation, ExecutionDependencyRelation::Imports) && only == "imports"
                    || matches!(relation, ExecutionDependencyRelation::Reads) && only == "reads"
                    || matches!(relation, ExecutionDependencyRelation::Uses) && only == "uses")
            },
        );
        parse_info.execution_dependents.retain(
            |ExecutionDependent {
                 dependent_relation: relation,
                 ..
             }| {
                !(matches!(relation, ExecutionDependentRelation::Assigns) && only == "assigns"
                    || matches!(relation, ExecutionDependentRelation::Alters) && only == "alters"
                    || matches!(relation, ExecutionDependentRelation::Declares)
                        && only == "declares"
                    || matches!(relation, ExecutionDependentRelation::Writes) && only == "writes")
            },
        )
    }

    parse_info.execution_dependencies.append(&mut dependencies);
    parse_info.execution_dependents.append(&mut dependents);
}

/// Parse a tag from a comment line
fn parse_tag(line: &str) -> Option<ExecutionTag> {
    static REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(@global\s+)?@([a-zA-Z]+)\s+(.*?)?\s*(:?\*/)?$")
            .expect("Unable to create regex")
    });

    REGEX.captures(line).map(|captures| {
        let is_global = captures.get(1).is_some();
        let name = captures[2].to_lowercase();
        let value = captures
            .get(3)
            .map_or_else(String::new, |group| group.as_str().to_string());
        ExecutionTag {
            name,
            value,
            is_global,
            ..Default::default()
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_tag() {
        assert_eq!(
            parse_tag("-- @db sqlite://some/file.db3"),
            Some(ExecutionTag {
                name: "db".to_string(),
                value: "sqlite://some/file.db3".to_string(),
                is_global: false,
                ..Default::default()
            })
        );
        assert_eq!(
            parse_tag("-- @global @db postgres://user:pwd@host:5432/db"),
            Some(ExecutionTag {
                name: "db".to_string(),
                value: "postgres://user:pwd@host:5432/db".to_string(),
                is_global: true,
                ..Default::default()
            })
        );
    }
}
