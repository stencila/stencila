//! Shared graph evidence constructors.
//!
//! Evidence labels are used across document, workspace, code, and environment
//! graph extraction. Keeping the constructors in one place prevents provenance
//! labels and default metadata from drifting between collectors.

use stencila_schema::{GraphEvidence, GraphEvidenceKind};

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
