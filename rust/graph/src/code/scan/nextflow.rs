use std::collections::BTreeSet;

use super::super::{
    facts::{CodeFacts, WorkflowResourceKind},
    util::{collect_string_literals, is_identifier_like},
};

/// Collect Nextflow facts from source text.
///
/// The current Nextflow tree-sitter Rust binding is not mature enough to link
/// directly here, so this pass recognizes the stable surface syntax that drives
/// graph semantics: `process` declarations, `input` and `output` path literals,
/// and execution blocks. It intentionally ignores dynamic channels and Groovy
/// expressions because their concrete resources are runtime values.
pub(in crate::code) fn collect_nextflow_text_facts(source: &str, facts: &mut CodeFacts) {
    let mut current_process = None::<String>;
    let mut current_block = None::<NextflowBlock>;
    let mut brace_depth = 0isize;
    let mut offset = 0usize;

    for raw_line in source.lines() {
        let line = raw_line.trim();
        let mut started_process = false;

        if let Some(process) = nextflow_process_name(line) {
            facts.record_workflow_rule(
                process.clone(),
                Some(offset + raw_line.find(line).unwrap_or_default()),
            );
            current_process = Some(process);
            current_block = None;
            started_process = true;
        } else if let Some(process) = current_process.as_ref() {
            if let Some((block, rest)) = nextflow_block(line) {
                current_block = Some(block);
                match block {
                    NextflowBlock::Input => {
                        extend_nextflow_literals(facts, process, rest, WorkflowResourceKind::Read);
                    }
                    NextflowBlock::Output => {
                        extend_nextflow_literals(facts, process, rest, WorkflowResourceKind::Write);
                    }
                    NextflowBlock::Script | NextflowBlock::Shell | NextflowBlock::Exec => {
                        facts.record_workflow_call(Some(process), block.call_name());
                    }
                }
            } else if let Some(block) = current_block {
                match block {
                    NextflowBlock::Input => {
                        extend_nextflow_literals(facts, process, line, WorkflowResourceKind::Read);
                    }
                    NextflowBlock::Output => {
                        extend_nextflow_literals(facts, process, line, WorkflowResourceKind::Write);
                    }
                    NextflowBlock::Script | NextflowBlock::Shell | NextflowBlock::Exec => {}
                }
            }
        }

        if current_process.is_some() {
            let brace_delta =
                raw_line.matches('{').count() as isize - raw_line.matches('}').count() as isize;
            if started_process {
                brace_depth = brace_delta.max(1);
            } else {
                brace_depth += brace_delta;
                if brace_depth <= 0 {
                    current_process = None;
                    current_block = None;
                }
            }
        }

        offset += raw_line.len() + 1;
    }
}

/// Nextflow process sub-block that affects static graph facts.
///
/// Keeping the block state explicit lets the text scanner associate following
/// `path "..."` lines with either inputs or outputs until another block starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextflowBlock {
    /// Input declarations.
    Input,

    /// Output declarations.
    Output,

    /// Script execution block.
    Script,

    /// Shell execution block.
    Shell,

    /// Groovy execution block.
    Exec,
}

impl NextflowBlock {
    /// Return the call-like graph label for an executable block.
    fn call_name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
            Self::Script => "script",
            Self::Shell => "shell",
            Self::Exec => "exec",
        }
    }
}

/// Extract a simple Nextflow process name from a declaration line.
///
/// Process names can be bare or quoted. The scanner only accepts the first
/// static token after `process`, which avoids treating Groovy expressions as
/// concrete workflow rule ids.
fn nextflow_process_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("process ")?.trim();
    let rest = rest.trim_start_matches('(').trim_start();
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;

    if matches!(first, '\'' | '"') {
        let start = first.len_utf8();
        let end = rest[start..].find(first)? + start;
        let name = &rest[start..end];
        return is_identifier_like(name).then(|| name.to_string());
    }

    let end = rest
        .char_indices()
        .find_map(|(index, char)| {
            (char.is_whitespace() || matches!(char, '{' | '(')).then_some(index)
        })
        .unwrap_or(rest.len());
    let name = &rest[..end];
    is_identifier_like(name).then(|| name.to_string())
}

/// Detect a Nextflow process block header and return its trailing text.
fn nextflow_block(line: &str) -> Option<(NextflowBlock, &str)> {
    let (prefix, rest) = line.split_once(':')?;
    let block = match prefix.trim() {
        "input" => NextflowBlock::Input,
        "output" => NextflowBlock::Output,
        "script" => NextflowBlock::Script,
        "shell" => NextflowBlock::Shell,
        "exec" => NextflowBlock::Exec,
        _ => return None,
    };
    Some((block, rest.trim()))
}

/// Extend Nextflow whole-file and process-level resource facts.
///
/// Nextflow paths often interpolate channel values, so this helper applies an
/// extra `$` guard on top of the shared static-literal filter before accepting a
/// quoted path as a concrete resource.
fn extend_nextflow_literals(
    facts: &mut CodeFacts,
    process: &str,
    source: &str,
    kind: WorkflowResourceKind,
) {
    let mut literals = BTreeSet::<String>::new();
    collect_string_literals(source, &mut literals);
    literals.retain(|literal| !literal.contains('$'));

    facts.extend_workflow_resources(Some(process), kind, literals);
}
