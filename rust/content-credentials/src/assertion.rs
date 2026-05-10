//! Mapping from Stencila provenance snapshots to C2PA assertion payloads.
//!
//! The C2PA wire payload structs live in [`crate::schema`]. This module keeps
//! the transformation from Stencila's internal handoff API explicit so the
//! snapshot shape can evolve without accidentally changing the external
//! `org.stencila.provenance` schema.

use serde_json::Map;

use crate::{
    schema::{
        ActivityRecord, AgentRecord, AiContentProfileRecord, AiDisclosureRecord, AssetRecord,
        AttributionRecord, DefinitionRecord, DependencyRecord, DisclosureAssessmentRecord,
        EnvironmentRecord, ExecutionDigestsRecord, ExecutionMessageRecord, ExecutionRecord,
        FileDigestRecord, IdentifierRecord, KernelRecord, NodeRecord, PROVENANCE_SCHEMA,
        PrivacyRecord, ProducerRecord, ProvenanceAssertion, ProvenanceCategoryRecord,
        ProvenanceRecord, RedactionRecord, ReproducibilityRecord, RuntimeRecord, SourceRecord,
        WorkflowRecord,
    },
    snapshot::{
        ActivitySnapshot, AgentSnapshot, AiDisclosureSnapshot, AssetSnapshot, AttributionSnapshot,
        DefinitionSnapshot, DependencySnapshot, DisclosureAssessmentSnapshot, DocumentSnapshot,
        EnvironmentSnapshot, ExecutionDigestSnapshot, ExecutionMessageSnapshot, ExecutionSnapshot,
        FileDigestSnapshot, IdentifierSnapshot, KernelSnapshot, PrivacySnapshot, ProducerSnapshot,
        ProvenanceCategorySnapshot, ProvenanceSnapshot, ProvenanceSummarySnapshot,
        RedactionSnapshot, ReproducibilitySnapshot, RuntimeSnapshot, SourceSnapshot,
        WorkflowSnapshot,
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
            asset,
            root_node,
            executed_node,
            output_node,
            activity,
            producer,
            attributions,
            source,
            execution,
            workflow,
            environment,
            ai_disclosure,
            provenance_summary,
            reproducibility,
            privacy,
        } = snapshot;

        let activities =
            activities_from_snapshots(activity, execution.as_ref(), workflow.is_some());

        Self {
            schema: PROVENANCE_SCHEMA.to_string(),
            version: 1,
            producer: producer.map_or_else(ProducerRecord::default, ProducerRecord::from),
            root_node: NodeRecord::from(root_node),
            executed_node: executed_node.map(NodeRecord::from),
            output_node: output_node.map(NodeRecord::from),
            asset: AssetRecord::from(asset),
            activities,
            attributions: attributions
                .into_iter()
                .map(AttributionRecord::from)
                .collect(),
            source: source.map(SourceRecord::from),
            execution: execution.map(ExecutionRecord::from),
            workflow: workflow.map(WorkflowRecord::from),
            environment: environment.map(EnvironmentRecord::from),
            ai_disclosure: ai_disclosure.map(AiDisclosureRecord::from),
            provenance: provenance_summary.map(ProvenanceRecord::from),
            reproducibility: reproducibility
                .map_or_else(ReproducibilityRecord::default, ReproducibilityRecord::from),
            privacy: privacy.map_or_else(PrivacyRecord::default, PrivacyRecord::from),
            extra: Map::new(),
        }
    }

    /// Whether this payload version is one this build understands.
    #[must_use]
    pub fn is_known_schema(&self) -> bool {
        self.version == 1
    }
}

impl Default for ProducerRecord {
    fn default() -> Self {
        Self {
            name: "Stencila".to_string(),
            version: stencila_version::STENCILA_VERSION.to_string(),
            stencila_schema_version: None,
            codec: None,
            renderer: None,
            extra: Map::new(),
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
            stencila_schema_version: snapshot.stencila_schema_version,
            codec: snapshot.codec,
            renderer: snapshot.renderer,
            ..Self::default()
        }
    }
}

impl From<AssetSnapshot> for AssetRecord {
    fn from(snapshot: AssetSnapshot) -> Self {
        Self {
            id: snapshot.id,
            asset_type: lower_kebab_value(&snapshot.kind),
            role: snapshot.role.map(|role| lower_kebab_value(&role)),
            media_type: snapshot.media_type,
            content_digest: snapshot.content_digest,
            label: snapshot.label,
            title: snapshot.title,
            size: snapshot.size,
            width: snapshot.width,
            height: snapshot.height,
            ..Self::default()
        }
    }
}

impl From<DocumentSnapshot> for NodeRecord {
    fn from(snapshot: DocumentSnapshot) -> Self {
        Self {
            node_type: snapshot.node_type,
            node_id: snapshot.node_id,
            node_path: snapshot.node_path,
            label_type: snapshot
                .label_type
                .map(|label_type| lower_kebab_value(&label_type)),
            label: snapshot.label,
            title: snapshot.title,
            programming_language: snapshot
                .programming_language
                .map(|language| lower_kebab_value(&language)),
            content_url: snapshot.content_url,
            media_type: snapshot.media_type,
            ..Self::default()
        }
    }
}

impl From<ActivitySnapshot> for ActivityRecord {
    fn from(snapshot: ActivitySnapshot) -> Self {
        Self {
            id: snapshot.id,
            activity_type: snapshot
                .kind
                .map_or_else(|| "sign".to_string(), |kind| lower_kebab_value(&kind)),
            name: snapshot.name,
            started_at: snapshot.started_at,
            ended_at: snapshot.ended_at,
            duration_ms: snapshot.duration_ms,
            associated_attribution_ids: snapshot.associated_attribution_ids,
            used_node_ids: snapshot.used_node_ids,
            generated_node_ids: snapshot.generated_node_ids,
            used_asset_ids: snapshot.used_asset_ids,
            generated_asset_ids: snapshot.generated_asset_ids,
            ..Self::default()
        }
    }
}

impl From<AttributionSnapshot> for AttributionRecord {
    fn from(snapshot: AttributionSnapshot) -> Self {
        Self {
            id: snapshot.id,
            agent: AgentRecord::from(snapshot.agent),
            role_name: snapshot.role_name.map(|role| lower_kebab_value(&role)),
            format: snapshot.format.map(|format| lower_kebab_value(&format)),
            last_modified: snapshot.last_modified,
            scope: snapshot.scope.map(|scope| lower_kebab_value(&scope)),
            provenance_category: snapshot
                .provenance_category
                .map(|category| lower_kebab_value(&category)),
            character_count: snapshot.character_count,
            character_percent: snapshot.character_percent,
            ..Self::default()
        }
    }
}

impl From<AgentSnapshot> for AgentRecord {
    fn from(snapshot: AgentSnapshot) -> Self {
        Self {
            agent_type: snapshot.kind.map(|kind| lower_kebab_value(&kind)),
            name: snapshot.name,
            id: snapshot.id,
            identifiers: snapshot
                .identifiers
                .into_iter()
                .map(IdentifierRecord::from)
                .collect(),
            provider: snapshot.provider,
            version: snapshot.version,
            model: snapshot.model,
            model_identifier: snapshot.model_identifier,
            url: snapshot.url,
            ..Self::default()
        }
    }
}

impl From<IdentifierSnapshot> for IdentifierRecord {
    fn from(snapshot: IdentifierSnapshot) -> Self {
        Self {
            identifier_type: snapshot.kind.map(|kind| lower_kebab_value(&kind)),
            value: snapshot.value,
            ..Self::default()
        }
    }
}

impl From<ExecutionDigestSnapshot> for ExecutionDigestsRecord {
    fn from(snapshot: ExecutionDigestSnapshot) -> Self {
        Self {
            state: snapshot.state_digest,
            semantic: snapshot.semantic_digest,
            dependencies: snapshot.dependencies_digest,
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
            patch_digest: snapshot.patch_digest,
            tag: snapshot.tag,
            ..Self::default()
        }
    }
}

impl From<ExecutionSnapshot> for ExecutionRecord {
    fn from(snapshot: ExecutionSnapshot) -> Self {
        Self {
            status: snapshot.status.map(|status| lower_kebab_value(&status)),
            ended_at: snapshot.ended_at,
            duration_ms: snapshot.duration_ms,
            digests: snapshot.digest.map(ExecutionDigestsRecord::from),
            count: snapshot.count,
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
            language: snapshot
                .language
                .map(|language| lower_kebab_value(&language)),
            ..Self::default()
        }
    }
}

impl From<DependencySnapshot> for DependencyRecord {
    fn from(snapshot: DependencySnapshot) -> Self {
        Self {
            node_id: snapshot.node_id,
            node_type: snapshot.node_type,
            relation: snapshot
                .relation
                .map(|relation| lower_kebab_value(&relation)),
            digest: snapshot.digest,
            ..Self::default()
        }
    }
}

impl From<ExecutionMessageSnapshot> for ExecutionMessageRecord {
    fn from(snapshot: ExecutionMessageSnapshot) -> Self {
        Self {
            level: snapshot.level.map(|level| lower_kebab_value(&level)),
            error_type: snapshot
                .error_type
                .map(|error_type| lower_kebab_value(&error_type)),
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
            ..Self::default()
        }
    }
}

impl From<DefinitionSnapshot> for DefinitionRecord {
    fn from(snapshot: DefinitionSnapshot) -> Self {
        Self {
            definition_type: snapshot.kind.map(|kind| lower_kebab_value(&kind)),
            name: snapshot.name,
            role: snapshot.role.map(|role| lower_kebab_value(&role)),
            source_path: snapshot.source_path,
            version: snapshot.version,
            content_digest: snapshot.content_digest,
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
            digest: snapshot.digest,
            ..Self::default()
        }
    }
}

impl From<AiDisclosureSnapshot> for AiDisclosureRecord {
    fn from(snapshot: AiDisclosureSnapshot) -> Self {
        Self {
            model_type: lower_kebab_value(&snapshot.model_type),
            model_name: snapshot.model_name,
            model_identifier: snapshot.model_identifier,
            content_profile: snapshot.human_oversight_level.map(|human_oversight_level| {
                AiContentProfileRecord {
                    human_oversight_level: Some(lower_kebab_value(&human_oversight_level)),
                    ..Default::default()
                }
            }),
            scientific_domain: snapshot.scientific_domains,
            standard_assertion: snapshot.standard_assertion,
            ..Self::default()
        }
    }
}

impl From<ProvenanceSummarySnapshot> for ProvenanceRecord {
    fn from(snapshot: ProvenanceSummarySnapshot) -> Self {
        Self {
            basis: snapshot.basis.map(|basis| lower_kebab_value(&basis)),
            human_percent: snapshot.human_percent,
            machine_percent: snapshot.machine_percent,
            ai_assisted_percent: snapshot.ai_assisted_percent,
            source: snapshot.source,
            source_version: snapshot.source_version,
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
            category: lower_kebab_value(&snapshot.provenance_category),
            character_count: snapshot.character_count,
            character_percent: snapshot.character_percent,
            ..Self::default()
        }
    }
}

impl Default for ReproducibilityRecord {
    fn default() -> Self {
        Self {
            status: "not-checked".to_string(),
            policy: None,
            checked_by: None,
            checked_at: None,
            comparison: None,
            extra: Map::new(),
        }
    }
}

impl From<ReproducibilitySnapshot> for ReproducibilityRecord {
    fn from(snapshot: ReproducibilitySnapshot) -> Self {
        Self {
            status: lower_kebab_value(&snapshot.reproducibility_status),
            policy: snapshot.policy,
            checked_by: snapshot.checked_by,
            checked_at: snapshot.checked_at,
            comparison: snapshot.comparison,
            ..Self::default()
        }
    }
}

impl Default for DisclosureAssessmentRecord {
    fn default() -> Self {
        Self {
            status: "not-assessed".to_string(),
            policy: None,
            assessed_at: None,
            extra: Map::new(),
        }
    }
}

impl From<DisclosureAssessmentSnapshot> for DisclosureAssessmentRecord {
    fn from(snapshot: DisclosureAssessmentSnapshot) -> Self {
        Self {
            status: snapshot.status,
            policy: snapshot.policy,
            assessed_at: snapshot.assessed_at,
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
            personal_data: DisclosureAssessmentRecord::from(snapshot.personal_data),
            secrets: DisclosureAssessmentRecord::from(snapshot.secrets),
            ..Self::default()
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

fn activities_from_snapshots(
    activity: Option<ActivitySnapshot>,
    execution: Option<&ExecutionSnapshot>,
    has_workflow: bool,
) -> Vec<ActivityRecord> {
    let mut activities = Vec::new();

    if let Some(execution) = execution {
        activities.push(execution_activity(execution, activity.as_ref()));
    }

    if let Some(activity) = activity {
        let record = ActivityRecord::from(activity);
        if record.activity_type != "execute" || activities.is_empty() {
            activities.push(record);
        }
    }

    if activities.is_empty() {
        activities.push(default_activity(has_workflow));
    }

    activities
}

fn execution_activity(
    execution: &ExecutionSnapshot,
    related_activity: Option<&ActivitySnapshot>,
) -> ActivityRecord {
    ActivityRecord {
        activity_type: "execute".to_string(),
        name: Some("Execute node".to_string()),
        ended_at: execution.ended_at.clone(),
        duration_ms: execution.duration_ms,
        generated_node_ids: related_activity
            .map(|activity| activity.generated_node_ids.clone())
            .filter(|ids| !ids.is_empty())
            .or_else(|| related_activity.map(|activity| activity.used_node_ids.clone()))
            .unwrap_or_default(),
        ..Default::default()
    }
}

fn default_activity(has_workflow: bool) -> ActivityRecord {
    let activity_type = if has_workflow { "run" } else { "sign" };

    ActivityRecord {
        activity_type: activity_type.to_string(),
        ..Default::default()
    }
}

fn lower_kebab_value(value: &str) -> String {
    let trimmed = value.trim();
    let mut result = String::with_capacity(trimmed.len());
    let chars = trimmed.chars().collect::<Vec<_>>();

    for (index, ch) in chars.iter().copied().enumerate() {
        let prev = index
            .checked_sub(1)
            .and_then(|prev| chars.get(prev))
            .copied();
        let next = chars.get(index + 1).copied();

        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() {
                let boundary = prev.is_some_and(|prev| {
                    prev.is_ascii_lowercase()
                        || prev.is_ascii_digit()
                        || (prev.is_ascii_uppercase()
                            && next.is_some_and(|next| next.is_ascii_lowercase()))
                });
                if boundary && !result.ends_with('-') && !result.ends_with('.') {
                    result.push('-');
                }
                result.push(ch.to_ascii_lowercase());
            } else {
                result.push(ch.to_ascii_lowercase());
            }
        } else if ch == '.' {
            if !result.ends_with('.') && !result.ends_with('-') {
                result.push('.');
            }
        } else if !result.is_empty() && !result.ends_with('-') && !result.ends_with('.') {
            result.push('-');
        }
    }

    while result.ends_with('-') || result.ends_with('.') {
        result.pop();
    }

    result
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

        assert_eq!(parsed.schema, PROVENANCE_SCHEMA);
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.producer.name, "Stencila");
        assert_eq!(parsed.asset.media_type, "image/png");
        assert_eq!(parsed.asset.content_digest, "sha256:abc");
        assert_eq!(parsed.asset.asset_type, "image");
        assert_eq!(parsed.root_node.node_type, "File");
        assert!(parsed.executed_node.is_none());
        assert!(parsed.output_node.is_none());
        assert_eq!(parsed.activities[0].activity_type, "sign");
        assert_eq!(parsed.reproducibility.status, "not-checked");
        assert_eq!(parsed.privacy.personal_data.status, "not-assessed");
        assert_eq!(parsed.privacy.secrets.status, "not-assessed");
        assert!(parsed.is_known_schema());
        assert!(parsed.extra.is_empty());
    }

    /// Ensures future assertion fields survive deserialization.
    #[test]
    fn unknown_fields_preserved() {
        // A future payload includes fields this build does not know about.
        let raw = json!({
            "schema": PROVENANCE_SCHEMA,
            "version": 1,
            "producer": { "name": "Stencila", "version": "9.9.9" },
            "asset": {
                "assetType": "figure",
                "mediaType": "image/png",
                "contentDigest": "sha256:abc",
                "assetFuture": "kept"
            },
            "rootNode": { "nodeType": "CodeChunk" },
            "activities": [{ "activityType": "execute" }],
            "reproducibility": { "status": "not-checked" },
            "privacy": {
                "personalData": { "status": "not-assessed" },
                "secrets": { "status": "not-assessed" }
            },
            "newField": "future",
            "nested": { "more": [1, 2, 3] }
        });

        let parsed: ProvenanceAssertion = serde_json::from_value(raw.clone()).expect("deserialize");
        assert_eq!(parsed.extra.get("newField"), Some(&json!("future")));
        assert!(parsed.extra.contains_key("nested"));
        assert_eq!(parsed.asset.extra.get("assetFuture"), Some(&json!("kept")));

        let again = serde_json::to_value(&parsed).expect("serialize");
        assert_eq!(again.get("newField"), Some(&json!("future")));
        assert_eq!(again.get("nested"), Some(&json!({ "more": [1, 2, 3] })));
        assert_eq!(
            again
                .get("asset")
                .and_then(|asset| asset.get("assetFuture")),
            Some(&json!("kept"))
        );
    }

    /// Ensures executable exports record execution before export in the activity list.
    #[test]
    fn executable_export_records_ordered_activities() {
        let assertion = ProvenanceAssertion::from_snapshot(ProvenanceSnapshot {
            asset: AssetSnapshot::new("Image", "image/png", "sha256:abc"),
            executed_node: Some(DocumentSnapshot {
                node_type: "CodeChunk".to_string(),
                node_id: Some("chunk-1".to_string()),
                ..Default::default()
            }),
            output_node: Some(DocumentSnapshot {
                node_type: "ImageObject".to_string(),
                node_id: Some("image-1".to_string()),
                content_url: Some("figures/example.png".to_string()),
                media_type: Some("image/png".to_string()),
                ..Default::default()
            }),
            activity: Some(ActivitySnapshot {
                kind: Some("Export".to_string()),
                used_node_ids: vec!["image-1".to_string()],
                generated_asset_ids: vec!["exported-asset".to_string()],
                ..Default::default()
            }),
            execution: Some(ExecutionSnapshot {
                status: Some("Succeeded".to_string()),
                ended_at: Some("2026-05-10T01:12:40Z".to_string()),
                duration_ms: Some(2110),
                ..Default::default()
            }),
            ..ProvenanceSnapshot::for_asset(AssetSnapshot::new("Image", "image/png", "sha256:abc"))
        });

        let activity_types = assertion
            .activities
            .iter()
            .map(|activity| activity.activity_type.as_str())
            .collect::<Vec<_>>();

        assert_eq!(activity_types, ["execute", "export"]);
        assert_eq!(assertion.activities[0].duration_ms, Some(2110));
        assert_eq!(assertion.activities[0].generated_node_ids, ["image-1"]);
        assert_eq!(assertion.activities[1].used_node_ids, ["image-1"]);
        assert_eq!(
            assertion.activities[1].generated_asset_ids,
            ["exported-asset"]
        );
        assert_eq!(
            assertion.execution.and_then(|execution| execution.status),
            Some("succeeded".to_string())
        );
    }

    /// Ensures asset content digests deserialize with the current field name.
    #[test]
    fn asset_content_digest_deserializes() {
        let raw = json!({
            "assetType": "image",
            "mediaType": "image/png",
            "contentDigest": "sha256:abc"
        });

        let parsed: AssetRecord = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(parsed.content_digest, "sha256:abc");
    }

    /// Ensures compatible v1 schema URLs are accepted but incompatible versions are not.
    #[test]
    fn schema_compatibility_uses_payload_version() {
        let mut a = ProvenanceAssertion::new_v1("image/png", "sha256:abc");
        a.schema =
            "https://stencila.org/stencila-provenance-assertion-v1.1.schema.json".to_string();
        assert!(a.is_known_schema());

        a.version = 2;
        assert!(!a.is_known_schema());
    }
}
