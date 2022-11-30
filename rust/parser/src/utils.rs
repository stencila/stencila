//! Utility functions for use by parser implementations

use std::path::Path;

use stencila_schema::{
    ExecutionDependency, ExecutionDependencyNode, ExecutionDependencyRelation, ExecutionDependent,
    ExecutionDependentNode, ExecutionDependentRelation, File, SoftwareSourceCode, Variable,
};

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

/// Create an [`ExecutionDependent`] reflecting assignment of a variable
pub fn assigns_variable(
    name: &str,
    path: Option<&Path>,
    kind: Option<String>,
    code_location: Option<[usize; 4]>,
) -> ExecutionDependent {
    ExecutionDependent {
        dependent_relation: ExecutionDependentRelation::Assigns,
        dependent_node: dependent_variable(name, path, kind),
        code_location,
        ..Default::default()
    }
}

/// Create an [`ExecutionDependent`] reflecting declaration of a variable
pub fn declares_variable(
    name: &str,
    path: Option<&Path>,
    kind: Option<String>,
    code_location: Option<[usize; 4]>,
) -> ExecutionDependent {
    ExecutionDependent {
        dependent_relation: ExecutionDependentRelation::Declares,
        dependent_node: dependent_variable(name, path, kind),
        code_location,
        ..Default::default()
    }
}

/// Create an [`ExecutionDependent`] reflecting alteration of a variable
pub fn alters_variable(
    name: &str,
    path: Option<&Path>,
    kind: Option<String>,
    code_location: Option<[usize; 4]>,
) -> ExecutionDependent {
    ExecutionDependent {
        dependent_relation: ExecutionDependentRelation::Alters,
        dependent_node: dependent_variable(name, path, kind),
        code_location,
        ..Default::default()
    }
}

/// Create a [`ExecutionDependent`] for writing a file
pub fn writes_file(path: &Path, code_location: Option<[usize; 4]>) -> ExecutionDependent {
    ExecutionDependent {
        dependent_relation: ExecutionDependentRelation::Writes,
        dependent_node: ExecutionDependentNode::File(File {
            path: path.to_string_lossy().to_string(),
            ..Default::default()
        }),
        code_location,
        ..Default::default()
    }
}

/// Create an [`ExecutionDependency`] reflecting use of a variable
pub fn uses_variable(
    name: &str,
    path: Option<&Path>,
    kind: Option<String>,
    code_location: Option<[usize; 4]>,
) -> ExecutionDependency {
    ExecutionDependency {
        dependency_relation: ExecutionDependencyRelation::Uses,
        dependency_node: ExecutionDependencyNode::Variable(Variable {
            namespace: path
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default(),
            name: name.to_string(),
            kind: kind.map(Box::new),
            ..Default::default()
        }),
        code_location,
        ..Default::default()
    }
}

/// Create a [`ExecutionDependency`] for reading a file
pub fn reads_file(path: &Path, code_location: Option<[usize; 4]>) -> ExecutionDependency {
    ExecutionDependency {
        dependency_relation: ExecutionDependencyRelation::Reads,
        dependency_node: ExecutionDependencyNode::File(File {
            path: path.to_string_lossy().to_string(),
            ..Default::default()
        }),
        code_location,
        ..Default::default()
    }
}

/// Create a [`ExecutionDependency`] for importing a source file
pub fn imports_file(path: &Path, code_location: Option<[usize; 4]>) -> ExecutionDependency {
    ExecutionDependency {
        dependency_relation: ExecutionDependencyRelation::Imports,
        dependency_node: ExecutionDependencyNode::File(File {
            path: path.to_string_lossy().to_string(),
            ..Default::default()
        }),
        code_location,
        ..Default::default()
    }
}

/// Create a [`ExecutionDependency`] for importing a source code module
pub fn imports_module(name: &str, code_location: Option<[usize; 4]>) -> ExecutionDependency {
    ExecutionDependency {
        dependency_relation: ExecutionDependencyRelation::Imports,
        dependency_node: ExecutionDependencyNode::SoftwareSourceCode(SoftwareSourceCode {
            name: Some(Box::new(name.to_string())),
            ..Default::default()
        }),
        code_location,
        ..Default::default()
    }
}

/// Create a [`ExecutionDependentNode`] for a variable
pub fn dependent_variable(
    name: &str,
    path: Option<&Path>,
    kind: Option<String>,
) -> ExecutionDependentNode {
    ExecutionDependentNode::Variable(variable(name, path, kind))
}

/// Create a [`ExecutionDependencyNode`] for a variable
pub fn dependency_variable(
    name: &str,
    path: Option<&Path>,
    kind: Option<String>,
) -> ExecutionDependencyNode {
    ExecutionDependencyNode::Variable(variable(name, path, kind))
}

/// Create a variable
pub fn variable(name: &str, path: Option<&Path>, kind: Option<String>) -> Variable {
    Variable {
        namespace: path
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        name: name.to_string(),
        kind: kind.map(Box::new),
        ..Default::default()
    }
}

/// Remove `Uses` execution dependencies which has an `Assigns` in execution dependents
pub fn remove_uses_of_assigned(
    dependencies: &mut Vec<ExecutionDependency>,
    dependents: &[ExecutionDependent],
) {
    dependencies.retain(|dependency| {
        if let ExecutionDependency {
            dependency_relation: ExecutionDependencyRelation::Uses,
            dependency_node:
                ExecutionDependencyNode::Variable(Variable {
                    name: dependency_name,
                    ..
                }),
            ..
        } = dependency
        {
            !dependents.iter().any(|dependent| {
                if let ExecutionDependent {
                    dependent_relation: ExecutionDependentRelation::Assigns,
                    dependent_node:
                        ExecutionDependentNode::Variable(Variable {
                            name: dependent_name,
                            ..
                        }),
                    ..
                } = dependent
                {
                    dependency_name == dependent_name
                } else {
                    false
                }
            })
        } else {
            true
        }
    })
}
