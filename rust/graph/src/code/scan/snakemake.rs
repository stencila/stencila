use super::super::{
    facts::{
        CodeFacts, WorkflowResourceKind, WorkflowScriptExecutionFact, WorkflowScriptExecutionKind,
    },
    util::{clean_string_literal, collect_path_expressions, direct_python_script},
};

/// Collect Snakemake facts from source text as a grammar fallback.
///
/// Snakemake's tree-sitter grammar can identify rule and directive nodes, but
/// directive payloads are easier to normalize from the original indentation.
/// This fallback keeps input, output, script, and shell facts stable across small
/// grammar shape changes.
pub(in crate::code) fn collect_snakemake_text_facts(source: &str, facts: &mut CodeFacts) {
    let lines = source_lines(source);
    let mut current_rule = None::<String>;
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].text.trim();
        if let Some(rule_name) = line
            .strip_prefix("rule ")
            .and_then(|rest| rest.strip_suffix(':'))
        {
            let rule_name = rule_name.trim();
            if !rule_name.is_empty() {
                let rule_name = rule_name.to_string();
                facts.record_workflow_unit(rule_name.clone(), None);
                current_rule = Some(rule_name);
            }
        }

        for (directive, target) in [
            (
                "input:",
                SnakemakeDirective::Resource(WorkflowResourceKind::Read),
            ),
            (
                "output:",
                SnakemakeDirective::Resource(WorkflowResourceKind::Write),
            ),
            (
                "script:",
                SnakemakeDirective::Resource(WorkflowResourceKind::Script),
            ),
            ("shell:", SnakemakeDirective::Shell),
        ] {
            if line.starts_with(directive) {
                let text = collect_directive_text(&lines, index, directive);
                let offset = lines[index].offset + leading_spaces(lines[index].text);
                let mut paths = Default::default();
                collect_path_expressions(&text, &mut paths);
                match target {
                    SnakemakeDirective::Resource(kind) => {
                        facts.extend_workflow_resources(
                            current_rule.as_deref(),
                            kind,
                            paths,
                            Some(offset),
                        );
                    }
                    SnakemakeDirective::Shell => {
                        if let Some(command) = clean_string_literal(&text)
                            && let Some(path) = direct_python_script(&command)
                        {
                            facts.record_workflow_script(
                                current_rule.as_deref(),
                                WorkflowScriptExecutionFact {
                                    path,
                                    kind: WorkflowScriptExecutionKind::Shell,
                                    offset: Some(offset),
                                    command: Some(command),
                                },
                            );
                        }
                        facts.record_workflow_call(current_rule.as_deref(), "shell");
                    }
                }
            }
        }

        index += 1;
    }
}

/// One source line and its byte offset in the full source.
#[derive(Debug, Clone, Copy)]
struct SourceLine<'a> {
    text: &'a str,
    offset: usize,
}

/// Split source into lines while preserving their byte offsets.
fn source_lines(source: &str) -> Vec<SourceLine<'_>> {
    let mut offset = 0;
    source
        .split_inclusive('\n')
        .map(|line| {
            let current = offset;
            offset += line.len();
            let text = line.strip_suffix('\n').unwrap_or(line);
            let text = text.strip_suffix('\r').unwrap_or(text);
            SourceLine {
                text,
                offset: current,
            }
        })
        .collect()
}

/// Snakemake directive category used by the text fallback.
#[derive(Debug, Clone, Copy)]
enum SnakemakeDirective {
    /// Resource-bearing directive.
    Resource(WorkflowResourceKind),

    /// Shell execution directive.
    Shell,
}

/// Collect the text belonging to one Snakemake directive.
///
/// Directives can place literals on the same line or on following indented
/// lines. Using indentation mirrors Snakemake's source structure closely enough
/// for static literals while avoiding a grammar-specific payload visitor.
fn collect_directive_text(lines: &[SourceLine<'_>], start: usize, directive: &str) -> String {
    let mut text = lines[start]
        .text
        .trim()
        .strip_prefix(directive)
        .unwrap_or_default()
        .to_string();
    let base_indent = leading_spaces(lines[start].text);
    for line in lines.iter().skip(start + 1) {
        let trimmed = line.text.trim();
        if trimmed.is_empty() {
            continue;
        }
        let indent = leading_spaces(line.text);
        if indent <= base_indent || is_snakemake_directive_line(trimmed) {
            break;
        }
        text.push('\n');
        text.push_str(trimmed);
    }
    text
}

/// Check whether a trimmed Snakemake line starts a new directive.
///
/// The directive text collector uses this to stop before consuming the next
/// section of a rule block. The list contains directives that affect block
/// structure even when this pass does not extract facts from them.
fn is_snakemake_directive_line(line: &str) -> bool {
    line.split_once(':').is_some_and(|(prefix, _)| {
        matches!(
            prefix,
            "input"
                | "output"
                | "script"
                | "shell"
                | "params"
                | "log"
                | "benchmark"
                | "threads"
                | "resources"
                | "run"
        )
    }) || line.starts_with("rule ")
}

/// Count leading spaces in a source line.
///
/// Snakemake directive fallback parsing is indentation-sensitive, so it needs a
/// small helper that treats tabs and other characters conservatively.
fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|char| *char == ' ').count()
}
