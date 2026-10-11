//! Small parse-tree helpers shared by the I/O table and the resolution pass.
//!
//! Both passes need the same few answers about a call node: what is being
//! called, and which expression sits in each argument position or under each
//! keyword. Keeping those answers in one place stops the two passes from
//! drifting apart on shapes such as R's `argument` wrapper or Python's
//! keyword arguments.

use ast_grep_core::{Node, tree_sitter::StrDoc};

use super::language::CodeLanguage;

/// A parse-tree node in the source being analyzed.
pub(super) type SourceNode<'r> = Node<'r, StrDoc<CodeLanguage>>;

/// Arguments supplied at one call site.
pub(super) struct CallArguments<'r> {
    /// Positional arguments, in order.
    pub(super) positional: Vec<SourceNode<'r>>,

    /// Keyword arguments, by parameter name.
    pub(super) keyword: Vec<(String, SourceNode<'r>)>,
}

impl<'r> CallArguments<'r> {
    /// Return the argument bound to a parameter by position or by name.
    ///
    /// The keyword form wins, because a caller that names an argument has said
    /// which parameter it fills regardless of where it appears.
    pub(super) fn for_parameter(&self, index: usize, name: &str) -> Option<SourceNode<'r>> {
        self.keyword
            .iter()
            .find(|(keyword, _)| keyword == name)
            .map(|(_, node)| node.clone())
            .or_else(|| self.positional.get(index).cloned())
    }
}

/// Whether a node is a call in this language's grammar.
pub(super) fn is_call_node(node: &SourceNode<'_>) -> bool {
    matches!(node.kind().as_ref(), "call" | "call_expression")
}

/// Return the callee text of a call node.
pub(super) fn call_callee(node: &SourceNode<'_>) -> Option<String> {
    let callee = node
        .field("function")
        .or_else(|| node.children().find(|child| child.is_named()))?;
    Some(callee.text().trim().to_string())
}

/// Return the terminal name of a possibly qualified callee spelling.
///
/// `pd.read_csv`, `pandas.read_csv`, and `read_csv` all name the same API as
/// far as the I/O table is concerned; which module it came from is an import
/// fact, not an I/O fact.
///
/// R is qualified with `::` and uses `.` inside ordinary names, so `read.csv`
/// must survive intact while `terra::rast` reduces to `rast`.
pub(super) fn callee_name(language: CodeLanguage, callee: &str) -> &str {
    let separators: &[char] = if language == CodeLanguage::R {
        &[':']
    } else {
        &['.', ':']
    };
    callee
        .rsplit(separators)
        .find(|part| !part.is_empty())
        .unwrap_or(callee)
}

/// Collect the arguments supplied at a call site.
///
/// Returns `None` when the call spreads a collection or mapping into its
/// arguments, because which parameter receives which value is then unknown.
pub(super) fn call_argument_bindings<'r>(
    language: CodeLanguage,
    node: &SourceNode<'r>,
) -> Option<CallArguments<'r>> {
    let arguments = node.field("arguments").or_else(|| {
        node.children()
            .find(|child| matches!(child.kind().as_ref(), "argument_list" | "arguments"))
    })?;

    let mut collected = CallArguments {
        positional: Vec::new(),
        keyword: Vec::new(),
    };

    for child in arguments.children() {
        if !child.is_named() || child.kind() == "comment" {
            continue;
        }

        let kind = child.kind();
        if matches!(
            kind.as_ref(),
            "list_splat" | "dictionary_splat" | "spread_element"
        ) {
            return None;
        }

        if language == CodeLanguage::R {
            // R wraps every argument, named or not, in an `argument` node. The
            // grammar does not expose name and value as fields, so a named
            // argument is recognized by its `=` separator.
            if kind != "argument" {
                continue;
            }
            let named = child.children().any(|inner| inner.kind() == "=");
            let parts = child
                .children()
                .filter(|inner| inner.is_named())
                .collect::<Vec<_>>();
            let (name, value) = if named {
                (parts.first().cloned(), parts.last().cloned())
            } else {
                (None, parts.first().cloned())
            };
            let (Some(value), true) = (value, !named || parts.len() >= 2) else {
                continue;
            };
            match name {
                Some(name) => collected
                    .keyword
                    .push((name.text().trim().to_string(), value)),
                None => collected.positional.push(value),
            }
            continue;
        }

        if kind == "keyword_argument" {
            let (Some(name), Some(value)) = (child.field("name"), child.field("value")) else {
                return None;
            };
            collected
                .keyword
                .push((name.text().trim().to_string(), value));
            continue;
        }

        collected.positional.push(child);
    }

    Some(collected)
}

/// Return the positional argument expressions of a call node.
pub(super) fn call_arguments<'r>(
    language: CodeLanguage,
    node: &SourceNode<'r>,
) -> Vec<SourceNode<'r>> {
    call_argument_bindings(language, node)
        .map(|arguments| arguments.positional)
        .unwrap_or_default()
}
