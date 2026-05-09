//! Mapping from Stencila provenance snapshots to C2PA assertion payloads.
//!
//! The C2PA wire payload structs live in [`crate::schema`]. This module keeps
//! the transformation from Stencila's internal handoff API explicit so the
//! snapshot shape can evolve without accidentally changing the external
//! `org.stencila.provenance` schema.

#![expect(
    clippy::needless_update,
    reason = "struct update syntax keeps snapshot-to-schema mappings robust when optional wire fields are added"
)]

use serde_json::Map;

use crate::{
    schema::{
        AgentRecord, AssetRecord, DefinitionRecord, DependencyRecord, DocumentRecord,
        EnvironmentRecord, ExecutionDigestRecord, ExecutionMessageRecord, ExecutionRecord,
        FileDigestRecord, IoRecord, KernelRecord, PROVENANCE_SCHEMA_V1, PrivacyRecord,
        ProducerRecord, ProvenanceAssertion, ProvenanceCategoryRecord, ProvenanceSummaryRecord,
        RedactionRecord, RuntimeRecord, SkillUsageRecord, SourceRecord, VerificationRecord,
        WorkflowRecord,
    },
    snapshot::{
        AgentSnapshot, AssetSnapshot, DefinitionSnapshot, DependencySnapshot, DocumentSnapshot,
        EnvironmentSnapshot, ExecutionDigestSnapshot, ExecutionMessageSnapshot, ExecutionSnapshot,
        FileDigestSnapshot, IoSnapshot, KernelSnapshot, PrivacySnapshot, ProducerSnapshot,
        ProvenanceCategorySnapshot, ProvenanceSnapshot, ProvenanceSummarySnapshot,
        RedactionSnapshot, RuntimeSnapshot, SkillUsageSnapshot, SourceSnapshot,
        VerificationSnapshot, WorkflowSnapshot,
    },
};

impl ProvenanceAssertion {
    /// Construct a v1 assertion for an asset of the given media type and source digest.
    #[must_use]
    pub fn new_v1(media_type: impl Into<String>, source_digest: impl Into<String>) -> Self {
        let media_type = media_type.into();
        let source_digest = source_digest.into();
        let asset = AssetSnapshot::new(
            asset_kind_for_media_type(&media_type),
            media_type,
            source_digest,
        );
        Self::from_snapshot(ProvenanceSnapshot::for_asset(asset))
    }

    /// Construct a v1 assertion from Stencila's internal provenance snapshot.
    ///
    /// The snapshot is an internal aggregation API. This method performs the
    /// final projection into the schema-versioned assertion payload that is
    /// serialized into C2PA.
    #[must_use]
    pub fn from_snapshot(snapshot: ProvenanceSnapshot) -> Self {
        let ProvenanceSnapshot {
            profile,
            asset,
            document,
            producer,
            source,
            execution,
            workflow,
            environment,
            inputs,
            outputs,
            provenance_summary,
            verification,
            privacy,
        } = snapshot;

        let profile = profile.unwrap_or_else(|| default_profile(&asset.kind));

        Self {
            schema: PROVENANCE_SCHEMA_V1.to_string(),
            profile,
            producer: producer.map_or_else(ProducerRecord::default, ProducerRecord::from),
            asset: AssetRecord::from(asset),
            document: DocumentRecord::from(document),
            source: source.map(SourceRecord::from),
            execution: execution.map(ExecutionRecord::from),
            workflow: workflow.map(WorkflowRecord::from),
            environment: environment.map(EnvironmentRecord::from),
            inputs: inputs.into_iter().map(IoRecord::from).collect(),
            outputs: outputs.into_iter().map(IoRecord::from).collect(),
            provenance_summary: provenance_summary.map(ProvenanceSummaryRecord::from),
            verification: verification
                .map_or_else(VerificationRecord::default, VerificationRecord::from),
            privacy: privacy.map_or_else(PrivacyRecord::default, PrivacyRecord::from),
            extra: Map::new(),
        }
    }

    /// Whether this payload's schema URL is one this build understands.
    #[must_use]
    pub fn is_known_schema(&self) -> bool {
        self.schema == PROVENANCE_SCHEMA_V1
    }
}

impl Default for ProducerRecord {
    fn default() -> Self {
        Self {
            name: "Stencila".to_string(),
            version: stencila_version::STENCILA_VERSION.to_string(),
            schema_version: None,
            codec: None,
            renderer: None,
        }
    }
}

impl From<ProducerSnapshot> for ProducerRecord {
    fn from(snapshot: ProducerSnapshot) -> Self {
        Self {
            name: snapshot.name.unwrap_or_else(|| "Stencila".to_string()),
            version: snapshot
                .version
                .unwrap_or_else(|| stencila_version::STENCILA_VERSION.to_string()),
            schema_version: snapshot.schema_version,
            codec: snapshot.codec,
            renderer: snapshot.renderer,
            ..Self::default()
        }
    }
}

impl From<AssetSnapshot> for AssetRecord {
    fn from(snapshot: AssetSnapshot) -> Self {
        Self {
            kind: snapshot.kind,
            media_type: snapshot.media_type,
            digest: snapshot.digest,
            label: snapshot.label,
            title: snapshot.title,
            size: snapshot.size,
            width: snapshot.width,
            height: snapshot.height,
            ..Self::default()
        }
    }
}

impl From<DocumentSnapshot> for DocumentRecord {
    fn from(snapshot: DocumentSnapshot) -> Self {
        Self {
            node_type: snapshot.node_type,
            node_id: snapshot.node_id,
            node_path: snapshot.node_path,
            label_type: snapshot.label_type,
            label: snapshot.label,
            title: snapshot.title,
            programming_language: snapshot.programming_language,
            execution_digest: snapshot.execution_digest.map(ExecutionDigestRecord::from),
            ..Self::default()
        }
    }
}

impl From<ExecutionDigestSnapshot> for ExecutionDigestRecord {
    fn from(snapshot: ExecutionDigestSnapshot) -> Self {
        Self {
            state_digest: snapshot.state_digest,
            semantic_digest: snapshot.semantic_digest,
            dependencies_digest: snapshot.dependencies_digest,
            dependencies_stale: snapshot.dependencies_stale,
            dependencies_failed: snapshot.dependencies_failed,
            ..Self::default()
        }
    }
}

impl From<SourceSnapshot> for SourceRecord {
    fn from(snapshot: SourceSnapshot) -> Self {
        Self {
            repository: snapshot.repository,
            commit: snapshot.commit,
            path: snapshot.path,
            dirty: snapshot.dirty,
            patch_sha256: snapshot.patch_sha256,
            tag: snapshot.tag,
            ..Self::default()
        }
    }
}

impl From<ExecutionSnapshot> for ExecutionRecord {
    fn from(snapshot: ExecutionSnapshot) -> Self {
        Self {
            status: snapshot.status,
            ended: snapshot.ended,
            duration_ms: snapshot.duration_ms,
            execution_count: snapshot.execution_count,
            kernel: snapshot.kernel.map(KernelRecord::from),
            dependencies: snapshot
                .dependencies
                .into_iter()
                .map(DependencyRecord::from)
                .collect(),
            messages: snapshot
                .messages
                .into_iter()
                .map(ExecutionMessageRecord::from)
                .collect(),
            ..Self::default()
        }
    }
}

impl From<KernelSnapshot> for KernelRecord {
    fn from(snapshot: KernelSnapshot) -> Self {
        Self {
            name: snapshot.name,
            version: snapshot.version,
            language: snapshot.language,
            ..Self::default()
        }
    }
}

impl From<DependencySnapshot> for DependencyRecord {
    fn from(snapshot: DependencySnapshot) -> Self {
        Self {
            node_id: snapshot.node_id,
            node_type: snapshot.node_type,
            relation: snapshot.relation,
            digest: snapshot.digest,
            ..Self::default()
        }
    }
}

impl From<ExecutionMessageSnapshot> for ExecutionMessageRecord {
    fn from(snapshot: ExecutionMessageSnapshot) -> Self {
        Self {
            level: snapshot.level,
            error_type: snapshot.error_type,
            message: snapshot.message,
            ..Self::default()
        }
    }
}

impl From<WorkflowSnapshot> for WorkflowRecord {
    fn from(snapshot: WorkflowSnapshot) -> Self {
        Self {
            run_id: snapshot.run_id,
            workflow_name: snapshot.workflow_name,
            goal_digest: snapshot.goal_digest,
            node_id: snapshot.node_id,
            thread_id: snapshot.thread_id,
            artifact_id: snapshot.artifact_id,
            agent_session_id: snapshot.agent_session_id,
            agent: snapshot.agent.map(AgentRecord::from),
            definitions: snapshot
                .definitions
                .into_iter()
                .map(DefinitionRecord::from)
                .collect(),
            skill_usages: snapshot
                .skill_usages
                .into_iter()
                .map(SkillUsageRecord::from)
                .collect(),
            ..Self::default()
        }
    }
}

impl From<AgentSnapshot> for AgentRecord {
    fn from(snapshot: AgentSnapshot) -> Self {
        Self {
            name: snapshot.name,
            provider: snapshot.provider,
            model: snapshot.model,
            ..Self::default()
        }
    }
}

impl From<DefinitionSnapshot> for DefinitionRecord {
    fn from(snapshot: DefinitionSnapshot) -> Self {
        Self {
            kind: snapshot.kind,
            name: snapshot.name,
            source_path: snapshot.source_path,
            version: snapshot.version,
            content_hash: snapshot.content_hash,
            ..Self::default()
        }
    }
}

impl From<SkillUsageSnapshot> for SkillUsageRecord {
    fn from(snapshot: SkillUsageSnapshot) -> Self {
        Self {
            name: snapshot.name,
            source_path: snapshot.source_path,
            content_hash: snapshot.content_hash,
            loaded_by: snapshot.loaded_by,
            tool_call_id: snapshot.tool_call_id,
            turn_index: snapshot.turn_index,
            ..Self::default()
        }
    }
}

impl From<EnvironmentSnapshot> for EnvironmentRecord {
    fn from(snapshot: EnvironmentSnapshot) -> Self {
        Self {
            container_image: snapshot.container_image,
            os: snapshot.os,
            architecture: snapshot.architecture,
            runtimes: snapshot
                .runtimes
                .into_iter()
                .map(RuntimeRecord::from)
                .collect(),
            lockfiles: snapshot
                .lockfiles
                .into_iter()
                .map(FileDigestRecord::from)
                .collect(),
            ..Self::default()
        }
    }
}

impl From<RuntimeSnapshot> for RuntimeRecord {
    fn from(snapshot: RuntimeSnapshot) -> Self {
        Self {
            name: snapshot.name,
            version: snapshot.version,
            ..Self::default()
        }
    }
}

impl From<FileDigestSnapshot> for FileDigestRecord {
    fn from(snapshot: FileDigestSnapshot) -> Self {
        Self {
            path: snapshot.path,
            sha256: snapshot.sha256,
            ..Self::default()
        }
    }
}

impl From<IoSnapshot> for IoRecord {
    fn from(snapshot: IoSnapshot) -> Self {
        Self {
            kind: snapshot.kind,
            name: snapshot.name,
            uri: snapshot.uri,
            media_type: snapshot.media_type,
            digest: snapshot.digest,
            version: snapshot.version,
            access: snapshot.access,
            redaction: snapshot.redaction,
            size: snapshot.size,
            width: snapshot.width,
            height: snapshot.height,
            row_count: snapshot.row_count,
            column_count: snapshot.column_count,
            extra: snapshot.extra,
            ..Self::default()
        }
    }
}

impl From<ProvenanceSummarySnapshot> for ProvenanceSummaryRecord {
    fn from(snapshot: ProvenanceSummarySnapshot) -> Self {
        Self {
            human: snapshot.human,
            machine: snapshot.machine,
            ai_assisted: snapshot.ai_assisted,
            source: snapshot.source,
            schema_version: snapshot.schema_version,
            categories: snapshot
                .categories
                .into_iter()
                .map(ProvenanceCategoryRecord::from)
                .collect(),
            ..Self::default()
        }
    }
}

impl From<ProvenanceCategorySnapshot> for ProvenanceCategoryRecord {
    fn from(snapshot: ProvenanceCategorySnapshot) -> Self {
        Self {
            category: snapshot.category,
            character_count: snapshot.character_count,
            character_percent: snapshot.character_percent,
        }
    }
}

impl Default for VerificationRecord {
    fn default() -> Self {
        Self {
            reproducibility_status: "not-checked".to_string(),
            policy: None,
            verified_by: None,
            verified_at: None,
            comparison: None,
        }
    }
}

impl From<VerificationSnapshot> for VerificationRecord {
    fn from(snapshot: VerificationSnapshot) -> Self {
        Self {
            reproducibility_status: snapshot.reproducibility_status,
            policy: snapshot.policy,
            verified_by: snapshot.verified_by,
            verified_at: snapshot.verified_at,
            comparison: snapshot.comparison,
            ..Self::default()
        }
    }
}

impl From<PrivacySnapshot> for PrivacyRecord {
    fn from(snapshot: PrivacySnapshot) -> Self {
        Self {
            redactions: snapshot
                .redactions
                .into_iter()
                .map(RedactionRecord::from)
                .collect(),
            contains_personal_data: snapshot.contains_personal_data,
            contains_secrets: snapshot.contains_secrets,
        }
    }
}

impl From<RedactionSnapshot> for RedactionRecord {
    fn from(snapshot: RedactionSnapshot) -> Self {
        Self {
            field: snapshot.field,
            reason: snapshot.reason,
            ..Self::default()
        }
    }
}

fn default_profile(asset_kind: &str) -> String {
    match asset_kind {
        "figure" | "table" | "image" | "dataset" => "computational-output",
        "document" => "document-export",
        _ => "asset",
    }
    .to_string()
}

pub(crate) fn asset_kind_for_media_type(media_type: &str) -> &'static str {
    if media_type.starts_with("image/") {
        "image"
    } else if matches!(
        media_type,
        "application/pdf"
            | "text/html"
            | "text/markdown"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    ) {
        "document"
    } else if matches!(
        media_type,
        "text/csv" | "application/json" | "application/vnd.apache.parquet"
    ) {
        "dataset"
    } else {
        "asset"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Ensures the minimal v1 provenance assertion round-trips without changing known fields.
    #[test]
    fn round_trip_minimal() {
        let original = ProvenanceAssertion::new_v1("image/png", "sha256:abc");
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: ProvenanceAssertion = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.schema, PROVENANCE_SCHEMA_V1);
        assert_eq!(parsed.profile, "computational-output");
        assert_eq!(parsed.producer.name, "Stencila");
        assert_eq!(parsed.asset.media_type, "image/png");
        assert_eq!(parsed.asset.digest, "sha256:abc");
        assert_eq!(parsed.asset.kind, "image");
        assert_eq!(parsed.document.node_type, "File");
        assert_eq!(parsed.verification.reproducibility_status, "not-checked");
        assert!(parsed.is_known_schema());
        assert!(parsed.extra.is_empty());
    }

    /// Ensures future assertion fields survive deserialization.
    #[test]
    fn unknown_fields_preserved() {
        // A future payload includes fields this build does not know about.
        let raw = json!({
            "schema": PROVENANCE_SCHEMA_V1,
            "profile": "computational-output",
            "producer": { "name": "Stencila", "version": "9.9.9" },
            "asset": { "kind": "figure", "mediaType": "image/png", "digest": "sha256:abc" },
            "document": { "nodeType": "CodeChunk" },
            "verification": { "reproducibilityStatus": "not-checked" },
            "privacy": { "containsPersonalData": false, "containsSecrets": false },
            "newField": "future",
            "nested": { "more": [1, 2, 3] }
        });

        let parsed: ProvenanceAssertion = serde_json::from_value(raw.clone()).expect("deserialize");
        assert_eq!(parsed.extra.get("newField"), Some(&json!("future")));
        assert!(parsed.extra.contains_key("nested"));

        let again = serde_json::to_value(&parsed).expect("serialize");
        assert_eq!(again.get("newField"), Some(&json!("future")));
        assert_eq!(again.get("nested"), Some(&json!({ "more": [1, 2, 3] })));
    }

    /// Ensures the previous `sourceDigest` field name is still accepted for asset digests.
    #[test]
    fn legacy_source_digest_field_is_accepted() {
        let raw = json!({
            "kind": "image",
            "mediaType": "image/png",
            "sourceDigest": "sha256:abc"
        });

        let parsed: AssetRecord = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(parsed.digest, "sha256:abc");
    }

    /// Ensures schema URLs outside the current v1 URL are reported as unknown.
    #[test]
    fn unknown_schema_url_detected() {
        let mut a = ProvenanceAssertion::new_v1("image/png", "sha256:abc");
        a.schema = "https://stencila.org/v999/ProvenanceAssertion.schema.json".to_string();
        assert!(!a.is_known_schema());
    }
}
