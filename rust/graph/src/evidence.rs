//! Shared graph evidence constructors.
//!
//! Evidence labels are used across document, workspace, code, and environment
//! graph extraction. Keeping the constructors in one place prevents provenance
//! labels and default metadata from drifting between collectors.

use stencila_schema::{CodeLocation, GraphEvidence, GraphEvidenceKind};

/// Create evidence for an explicit authored relationship.
pub(crate) fn declared() -> GraphEvidence {
    evidence(GraphEvidenceKind::Declared)
}

/// Create evidence for a relationship resolved to a concrete graph node.
pub(crate) fn resolved() -> GraphEvidence {
    evidence(GraphEvidenceKind::Resolved)
}

/// Create evidence for a direct local or runtime observation.
pub(crate) fn observed() -> GraphEvidence {
    evidence(GraphEvidenceKind::Observed)
}

/// Create evidence for a deterministic relationship computed by Stencila.
pub(crate) fn computed() -> GraphEvidence {
    evidence(GraphEvidenceKind::Computed)
}

/// Create evidence for a relationship from persisted Stencila metadata.
pub(crate) fn recorded() -> GraphEvidence {
    evidence(GraphEvidenceKind::Recorded)
}

/// Create evidence for a relationship produced by static analysis.
pub(crate) fn static_analysis() -> GraphEvidence {
    evidence(GraphEvidenceKind::StaticAnalysis)
}

/// Create evidence for static analysis at a source offset.
pub(crate) fn static_analysis_at(
    source: &str,
    source_text: Option<&str>,
    offset: Option<usize>,
) -> GraphEvidence {
    let mut evidence = static_analysis();
    if let Some((source_text, offset)) = source_text.zip(offset) {
        evidence.code_location = Some(code_location(source, source_text, offset));
    }
    evidence
}

/// Create evidence for a declared reference that was also resolved locally.
pub(crate) fn declared_and_resolved() -> Vec<GraphEvidence> {
    vec![declared(), resolved()]
}

/// Create evidence for an observed relationship resolved to a concrete node.
pub(crate) fn observed_and_resolved() -> Vec<GraphEvidence> {
    vec![observed(), resolved()]
}

/// Build one evidence value.
fn evidence(kind: GraphEvidenceKind) -> GraphEvidence {
    GraphEvidence::new(kind)
}

/// Create a code location from a byte offset into source text.
fn code_location(source: &str, source_text: &str, offset: usize) -> CodeLocation {
    let offset = offset.min(source_text.len());
    let mut line = 0;
    let mut line_start = 0;

    for (index, char) in source_text.char_indices() {
        if index >= offset {
            break;
        }
        if char == '\n' {
            line += 1;
            line_start = index + char.len_utf8();
        }
    }

    let column = source_text.get(line_start..offset).map_or_else(
        || offset.saturating_sub(line_start) as u64,
        |text| text.chars().count() as u64,
    );

    CodeLocation {
        source: Some(source.to_string()),
        start_line: Some(line),
        start_column: Some(column),
        ..Default::default()
    }
}
