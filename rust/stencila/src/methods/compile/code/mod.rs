///! Functions etc for compiling programming language source code in `CodeChunk` and `CodeExpression` nodes.
use graphs::{Relation, Resource};
use std::path::Path;

#[allow(dead_code)]
mod utils;

#[cfg(feature = "tree-sitter")]
mod helpers;

#[cfg(feature = "compile-calc")]
pub mod calc;

#[cfg(feature = "compile-js")]
pub mod js;

#[cfg(feature = "compile-py")]
pub mod py;

#[cfg(feature = "compile-r")]
pub mod r;

#[cfg(feature = "compile-ts")]
pub mod ts;

/// Compile code in a particular language
#[allow(unused_variables)]
pub fn compile<P: AsRef<Path>>(path: P, code: &str, language: &str) -> Vec<(Relation, Resource)> {
    let path = path.as_ref();
    let pairs = match language {
        #[cfg(feature = "compile-calc")]
        "calc" => calc::compile(path, code),

        #[cfg(feature = "compile-js")]
        "js" | "javascript" => js::compile(path, code),

        #[cfg(feature = "compile-py")]
        "py" | "python" => py::compile(path, code),

        #[cfg(feature = "compile-r")]
        "r" => r::compile(path, code),

        #[cfg(feature = "compile-ts")]
        "ts" | "typescript" => ts::compile(path, code),

        _ => Vec::new(),
    };

    // Normalize pairs by removing any `Uses` of locally assigned variables
    let mut normalized: Vec<(Relation, Resource)> = Vec::with_capacity(pairs.len());
    for (relation, object) in pairs {
        let mut include = true;
        if matches!(relation, Relation::Use(..)) {
            for (other_relation, other_object) in &normalized {
                if matches!(other_relation, Relation::Assign(..)) && *other_object == object {
                    include = false;
                    break;
                }
            }
        }
        if !include {
            continue;
        }

        normalized.push((relation, object))
    }
    normalized
}
