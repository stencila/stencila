use std::path::{Path, PathBuf};

use common::{
    once_cell::sync::Lazy,
    regex::{Captures, Regex},
};
use stencila_schema::{
    CodeError, ExecutionDependency, ExecutionDependencyNode, ExecutionDependencyRelation, File,
};

/// Regex for detecting file interpolations within code
///
/// Only allows for @{file} pattern
pub static FILE_INTERP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:@\{([^}]+)\})").expect("Unable to create regex"));

/// Parse file interpolations in code into a vector of relations
///
/// Used by parsers to define the relations between some code and other resources.
pub fn parse_file_interps(code: &str, path: Option<&Path>) -> Vec<ExecutionDependency> {
    FILE_INTERP_REGEX
        .captures_iter(code)
        .map(|captures| {
            let file = captures.get(1).expect("Should always have one group");
            let path = path
                .map_or_else(
                    || PathBuf::from(file.as_str()),
                    |path| path.join(file.as_str()),
                )
                .to_string_lossy()
                .to_string();
            ExecutionDependency {
                dependency_relation: ExecutionDependencyRelation::Uses,
                dependency_node: ExecutionDependencyNode::File(File {
                    path,
                    ..Default::default()
                }),
                ..Default::default()
            }
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
