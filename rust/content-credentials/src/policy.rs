//! Privacy projection policy for Stencila content credentials.
//!
//! The policy layer is deliberately separate from the C2PA signing mechanics.
//! Callers may hand this crate rich local provenance snapshots, and this module
//! trims them to the selected credential profile before the payload is signed.

use std::{
    env,
    net::IpAddr,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    error::{Error, Result},
    schema::ProvenanceAssertion,
    snapshot::{
        ActivitySnapshot, AgentSnapshot, AssetSnapshot, DefinitionSnapshot, DependencySnapshot,
        DocumentSnapshot, EnvironmentSnapshot, ExecutionDigestSnapshot, ExecutionMessageSnapshot,
        ExecutionSnapshot, FileDigestSnapshot, KernelSnapshot, ProducerSnapshot,
        ProvenanceSnapshot, ProvenanceSummarySnapshot, RedactionSnapshot, ReproducibilitySnapshot,
        SourceSnapshot, WorkflowSnapshot,
    },
};

/// Preferred maximum size for the Stencila provenance assertion payload.
pub const ASSERTION_SOFT_SIZE_LIMIT: usize = 64 * 1024;

/// Maximum embedded Stencila provenance assertion payload size.
pub const ASSERTION_HARD_SIZE_LIMIT: usize = 1024 * 1024;

/// Redaction reason for omitted digests.
pub const REDACTION_DIGEST_OMITTED: &str = "digest-omitted";

/// Redaction reason for omitted URIs.
pub const REDACTION_URI_OMITTED: &str = "uri-omitted";

/// Redaction reason for redacted display names.
pub const REDACTION_NAME_REDACTED: &str = "name-redacted";

/// Redaction reason for records that are fully removed.
pub const REDACTION_FULLY_REDACTED: &str = "fully-redacted";

const REDACTED_NAME: &str = "redacted";

/// Content Credentials privacy projection profile.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CredentialProfile {
    /// Public-safe credential metadata.
    #[default]
    Public,

    /// More local detail for internal sharing.
    Private,

    /// Full local detail for controlled archives.
    Full,
}

impl CredentialProfile {
    /// Short label for this profile in CLI and `EncodeInfo` metadata.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Full => "full",
        }
    }

    /// Privacy policy identifier recorded in Stencila provenance assertions.
    #[must_use]
    pub const fn policy_name(self) -> &'static str {
        match self {
            Self::Public => "org.stencila.credentials.public.v1",
            Self::Private => "org.stencila.credentials.private.v1",
            Self::Full => "org.stencila.credentials.full.v1",
        }
    }
}

/// Concrete projection policy resolved for a signing request.
#[derive(Debug, Clone)]
pub struct ProjectionPolicy {
    profile: CredentialProfile,
    workspace_dir: Option<PathBuf>,
    home_dir: Option<PathBuf>,
}

impl ProjectionPolicy {
    /// Build a policy for the selected profile using the current process context.
    #[must_use]
    pub fn for_profile(profile: CredentialProfile) -> Self {
        Self {
            profile,
            workspace_dir: env::current_dir().ok(),
            home_dir: env::var_os("HOME").map(PathBuf::from),
        }
    }

    /// Build a policy with explicit path roots, useful for tests and embedders.
    #[must_use]
    pub fn with_roots(
        profile: CredentialProfile,
        workspace_dir: Option<PathBuf>,
        home_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            profile,
            workspace_dir,
            home_dir,
        }
    }

    /// Project a snapshot according to this policy.
    #[must_use]
    pub fn project_snapshot(&self, mut snapshot: ProvenanceSnapshot) -> ProvenanceSnapshot {
        let mut privacy = snapshot.privacy.take().unwrap_or_default();
        let mut redactions = std::mem::take(&mut privacy.redactions);

        self.project_asset(&mut snapshot.asset, &mut redactions);
        self.project_document(&mut snapshot.root_node, &mut redactions);
        self.project_node(
            "executedNode",
            snapshot.executed_node.as_mut(),
            &mut redactions,
        );
        self.project_node("outputNode", snapshot.output_node.as_mut(), &mut redactions);
        self.project_activity(snapshot.activity.as_mut(), &mut redactions);
        self.project_producer(snapshot.producer.as_mut(), &mut redactions);
        self.project_source(snapshot.source.as_mut(), &mut redactions);
        self.project_execution(snapshot.execution.as_mut(), &mut redactions);
        self.project_environment(snapshot.environment.as_mut(), &mut redactions);
        self.project_workflow(snapshot.workflow.as_mut(), &mut redactions);
        project_agents(
            "attributions",
            snapshot
                .attributions
                .iter_mut()
                .map(|attribution| &mut attribution.agent),
            self,
            &mut redactions,
        );

        if let Some(disclosure) = &mut snapshot.ai_disclosure {
            redact_secret_option(
                "aiDisclosure.modelName",
                &mut disclosure.model_name,
                &mut redactions,
            );
            redact_secret_option(
                "aiDisclosure.modelIdentifier",
                &mut disclosure.model_identifier,
                &mut redactions,
            );
        }

        self.project_provenance_summary(snapshot.provenance_summary.as_mut(), &mut redactions);
        self.project_reproducibility(snapshot.reproducibility.as_mut(), &mut redactions);

        privacy.redactions = redactions;
        privacy.personal_data.policy = Some(self.policy_name());
        privacy.secrets.policy = Some(self.policy_name());
        snapshot.privacy = Some(privacy);
        snapshot
    }

    /// Validate payload size after projection.
    ///
    /// This currently warns above the soft target and fails before the embedded
    /// hard cap. Externalized assertions can be added later without changing the
    /// policy boundary.
    ///
    /// # Errors
    ///
    /// Returns an error if the assertion cannot be serialized or is larger than
    /// [`ASSERTION_HARD_SIZE_LIMIT`].
    pub fn validate_assertion_size(&self, assertion: &ProvenanceAssertion) -> Result<usize> {
        let bytes = serde_json::to_vec(assertion)?;
        let len = bytes.len();

        if len > ASSERTION_HARD_SIZE_LIMIT {
            return Err(Error::other(format!(
                "content credentials assertion is {len} bytes, above the {ASSERTION_HARD_SIZE_LIMIT} byte embedded hard cap"
            )));
        }

        if len > ASSERTION_SOFT_SIZE_LIMIT {
            tracing::warn!(
                assertion_bytes = len,
                soft_limit = ASSERTION_SOFT_SIZE_LIMIT,
                "Content credentials assertion exceeds soft size target"
            );
        }

        Ok(len)
    }

    fn policy_name(&self) -> String {
        self.profile.policy_name().to_string()
    }

    fn project_asset(&self, asset: &mut AssetSnapshot, redactions: &mut Vec<RedactionSnapshot>) {
        redact_secret_option("asset.id", &mut asset.id, redactions);
        project_text_option(
            "asset.label",
            &mut asset.label,
            self.home_dir.as_deref(),
            redactions,
        );
        project_text_option(
            "asset.title",
            &mut asset.title,
            self.home_dir.as_deref(),
            redactions,
        );
    }

    fn project_document(
        &self,
        document: &mut DocumentSnapshot,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        redact_secret_option("rootNode.nodeId", &mut document.node_id, redactions);
        self.project_node_fields("rootNode", document, redactions);
    }

    fn project_node(
        &self,
        prefix: &str,
        node: Option<&mut DocumentSnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(node) = node else {
            return;
        };

        redact_secret_option(&format!("{prefix}.nodeId"), &mut node.node_id, redactions);
        self.project_path_option(
            &format!("{prefix}.nodePath"),
            &mut node.node_path,
            redactions,
        );
        self.project_node_fields(prefix, node, redactions);
    }

    fn project_node_fields(
        &self,
        prefix: &str,
        node: &mut DocumentSnapshot,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        project_text_option(
            &format!("{prefix}.label"),
            &mut node.label,
            self.home_dir.as_deref(),
            redactions,
        );
        project_text_option(
            &format!("{prefix}.title"),
            &mut node.title,
            self.home_dir.as_deref(),
            redactions,
        );
        redact_secret_option(
            &format!("{prefix}.programmingLanguage"),
            &mut node.programming_language,
            redactions,
        );
        self.project_uri_or_path_option(
            &format!("{prefix}.contentUrl"),
            &mut node.content_url,
            true,
            redactions,
        );
        redact_secret_option(
            &format!("{prefix}.mediaType"),
            &mut node.media_type,
            redactions,
        );
    }

    fn project_activity(
        &self,
        activity: Option<&mut ActivitySnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(activity) = activity else {
            return;
        };

        redact_secret_option("activity.id", &mut activity.id, redactions);
        redact_secret_option("activity.activityType", &mut activity.kind, redactions);
        project_text_option(
            "activity.name",
            &mut activity.name,
            self.home_dir.as_deref(),
            redactions,
        );
        project_string_list(
            "activity.associatedAttributionIds",
            &mut activity.associated_attribution_ids,
            self.home_dir.as_deref(),
            redactions,
        );
        project_string_list(
            "activity.usedNodeIds",
            &mut activity.used_node_ids,
            self.home_dir.as_deref(),
            redactions,
        );
        project_string_list(
            "activity.generatedNodeIds",
            &mut activity.generated_node_ids,
            self.home_dir.as_deref(),
            redactions,
        );
        project_string_list(
            "activity.usedAssetIds",
            &mut activity.used_asset_ids,
            self.home_dir.as_deref(),
            redactions,
        );
        project_string_list(
            "activity.generatedAssetIds",
            &mut activity.generated_asset_ids,
            self.home_dir.as_deref(),
            redactions,
        );
    }

    fn project_producer(
        &self,
        producer: Option<&mut ProducerSnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(producer) = producer else {
            return;
        };

        redact_secret_option("producer.name", &mut producer.name, redactions);
        redact_secret_option("producer.version", &mut producer.version, redactions);
        redact_secret_option(
            "producer.stencilaSchemaVersion",
            &mut producer.stencila_schema_version,
            redactions,
        );
        redact_secret_option("producer.codec", &mut producer.codec, redactions);
        project_text_option(
            "producer.renderer",
            &mut producer.renderer,
            self.home_dir.as_deref(),
            redactions,
        );
    }

    fn project_source(
        &self,
        source: Option<&mut SourceSnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(source) = source else {
            return;
        };

        if let Some(repository) = &mut source.repository {
            // Always strip URLs that carry secrets (basic auth, query
            // tokens). Under the Public profile, otherwise strip URLs
            // unless they live on a well-known public hosting domain — a
            // public GitHub URL is, by construction, already public.
            let omit_repository = has_secret(repository)
                || (self.profile == CredentialProfile::Public
                    && !is_public_hosting_url(repository));

            if omit_repository {
                source.repository = None;
                redactions.push(redaction("source.repository", REDACTION_URI_OMITTED));
            }
        }

        self.project_path_option("source.path", &mut source.path, redactions);

        if self.profile == CredentialProfile::Public
            && source.dirty == Some(true)
            && source.patch_digest.is_some()
        {
            source.patch_digest = None;
            redactions.push(redaction("source.patchDigest", REDACTION_DIGEST_OMITTED));
        }
    }

    fn project_execution(
        &self,
        execution: Option<&mut ExecutionSnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(execution) = execution else {
            return;
        };

        project_text_option(
            "execution.status",
            &mut execution.status,
            self.home_dir.as_deref(),
            redactions,
        );

        if let Some(kernel) = &mut execution.kernel {
            Self::project_kernel("execution.kernel", kernel, redactions);
        }

        if let Some(digest) = &mut execution.digest {
            project_execution_digest("execution.digests", digest, redactions);
        }

        for (index, dependency) in execution.dependencies.iter_mut().enumerate() {
            Self::project_dependency(
                &format!("execution.dependencies[{index}]"),
                dependency,
                redactions,
            );
        }

        for (index, message) in execution.messages.iter_mut().enumerate() {
            self.project_execution_message(
                &format!("execution.messages[{index}]"),
                message,
                redactions,
            );
        }
    }

    fn project_kernel(
        prefix: &str,
        kernel: &mut KernelSnapshot,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        redact_secret_option(&format!("{prefix}.name"), &mut kernel.name, redactions);
        redact_secret_option(
            &format!("{prefix}.version"),
            &mut kernel.version,
            redactions,
        );
        redact_secret_option(
            &format!("{prefix}.language"),
            &mut kernel.language,
            redactions,
        );
    }

    fn project_dependency(
        prefix: &str,
        dependency: &mut DependencySnapshot,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        redact_secret_option(
            &format!("{prefix}.nodeId"),
            &mut dependency.node_id,
            redactions,
        );
        redact_secret_option(
            &format!("{prefix}.nodeType"),
            &mut dependency.node_type,
            redactions,
        );
        redact_secret_option(
            &format!("{prefix}.relation"),
            &mut dependency.relation,
            redactions,
        );
        redact_secret_option(
            &format!("{prefix}.digest"),
            &mut dependency.digest,
            redactions,
        );
    }

    fn project_execution_message(
        &self,
        prefix: &str,
        message: &mut ExecutionMessageSnapshot,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        redact_secret_option(&format!("{prefix}.level"), &mut message.level, redactions);
        redact_secret_option(
            &format!("{prefix}.errorType"),
            &mut message.error_type,
            redactions,
        );
        project_text_option(
            &format!("{prefix}.message"),
            &mut message.message,
            self.home_dir.as_deref(),
            redactions,
        );
    }

    fn project_environment(
        &self,
        environment: Option<&mut EnvironmentSnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(environment) = environment else {
            return;
        };

        redact_secret_option(
            "environment.containerImage",
            &mut environment.container_image,
            redactions,
        );

        for (index, lockfile) in environment.lockfiles.iter_mut().enumerate() {
            self.project_file_digest(
                &format!("environment.lockfiles[{index}]"),
                lockfile,
                redactions,
            );
        }

        redact_secret_option("environment.os", &mut environment.os, redactions);
        redact_secret_option(
            "environment.architecture",
            &mut environment.architecture,
            redactions,
        );

        for (index, runtime) in environment.runtimes.iter_mut().enumerate() {
            redact_secret_option(
                &format!("environment.runtimes[{index}].name"),
                &mut runtime.name,
                redactions,
            );
            redact_secret_option(
                &format!("environment.runtimes[{index}].version"),
                &mut runtime.version,
                redactions,
            );
        }
    }

    fn project_workflow(
        &self,
        workflow: Option<&mut WorkflowSnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(workflow) = workflow else {
            return;
        };

        if self.profile == CredentialProfile::Public {
            redact_name_option(
                "workflow.workflowName",
                &mut workflow.workflow_name,
                redactions,
            );
            redact_secret_option("workflow.threadId", &mut workflow.thread_id, redactions);
            redact_secret_option(
                "workflow.agentSessionId",
                &mut workflow.agent_session_id,
                redactions,
            );
        }

        if let Some(agent) = &mut workflow.agent {
            self.project_agent("workflow.agent", agent, redactions);
        }

        for (index, definition) in workflow.definitions.iter_mut().enumerate() {
            self.project_definition(
                &format!("workflow.definitions[{index}]"),
                definition,
                redactions,
            );
        }
    }

    fn project_definition(
        &self,
        prefix: &str,
        definition: &mut DefinitionSnapshot,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        if self.profile == CredentialProfile::Public {
            redact_name_option(&format!("{prefix}.name"), &mut definition.name, redactions);
        }
        self.project_path_option(
            &format!("{prefix}.sourcePath"),
            &mut definition.source_path,
            redactions,
        );
        redact_secret_option(
            &format!("{prefix}.version"),
            &mut definition.version,
            redactions,
        );
    }

    fn project_file_digest(
        &self,
        prefix: &str,
        file: &mut FileDigestSnapshot,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        self.project_path_option(&format!("{prefix}.path"), &mut file.path, redactions);
        redact_secret_option(&format!("{prefix}.digest"), &mut file.digest, redactions);
    }

    fn project_uri_or_path_option(
        &self,
        field: &str,
        value: &mut Option<String>,
        allow_public_url: bool,
        redactions: &mut Vec<RedactionSnapshot>,
    ) -> bool {
        let Some(raw) = value.as_deref() else {
            return false;
        };

        if is_probable_url(raw) {
            if self.profile == CredentialProfile::Public
                && !(allow_public_url && public_web_url(raw))
            {
                *value = None;
                redactions.push(redaction(field, REDACTION_URI_OMITTED));
                return true;
            }
            return false;
        }

        self.project_path_option(field, value, redactions)
    }

    fn project_provenance_summary(
        &self,
        provenance_summary: Option<&mut ProvenanceSummarySnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(provenance_summary) = provenance_summary else {
            return;
        };

        project_text_option(
            "provenance.basis",
            &mut provenance_summary.basis,
            self.home_dir.as_deref(),
            redactions,
        );
        project_text_option(
            "provenance.source",
            &mut provenance_summary.source,
            self.home_dir.as_deref(),
            redactions,
        );
        redact_secret_option(
            "provenance.sourceVersion",
            &mut provenance_summary.source_version,
            redactions,
        );

        for (index, category) in provenance_summary.categories.iter_mut().enumerate() {
            redact_secret_string(
                &format!("provenance.categories[{index}].category"),
                &mut category.provenance_category,
                redactions,
            );
        }
    }

    fn project_reproducibility(
        &self,
        reproducibility: Option<&mut ReproducibilitySnapshot>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        let Some(reproducibility) = reproducibility else {
            return;
        };

        redact_secret_string(
            "reproducibility.status",
            &mut reproducibility.reproducibility_status,
            redactions,
        );
        redact_secret_option(
            "reproducibility.policy",
            &mut reproducibility.policy,
            redactions,
        );
        project_text_option(
            "reproducibility.checkedBy",
            &mut reproducibility.checked_by,
            self.home_dir.as_deref(),
            redactions,
        );

        if let Some(comparison) = &mut reproducibility.comparison {
            project_metadata(
                "reproducibility.comparison",
                comparison,
                self.home_dir.as_deref(),
                redactions,
            );
        }
    }

    fn project_path_option(
        &self,
        field: &str,
        value: &mut Option<String>,
        redactions: &mut Vec<RedactionSnapshot>,
    ) -> bool {
        let Some(path) = value.as_deref() else {
            return false;
        };

        if has_secret(path) {
            *value = None;
            redactions.push(redaction(field, REDACTION_URI_OMITTED));
            return true;
        }

        let path = Path::new(path);
        if !path.is_absolute() {
            return false;
        }

        if let Some(relative) = self.workspace_relative(path) {
            *value = Some(relative);
            return false;
        }

        if self.profile == CredentialProfile::Public && path.is_absolute() {
            *value = None;
            redactions.push(redaction(field, REDACTION_URI_OMITTED));
            return true;
        }

        false
    }

    fn workspace_relative(&self, path: &Path) -> Option<String> {
        let workspace_dir = self.workspace_dir.as_deref()?;
        let relative = path.strip_prefix(workspace_dir).ok()?;
        Some(relative.to_string_lossy().replace('\\', "/"))
    }

    fn project_agent(
        &self,
        prefix: &str,
        agent: &mut AgentSnapshot,
        redactions: &mut Vec<RedactionSnapshot>,
    ) {
        project_text_option(
            &format!("{prefix}.name"),
            &mut agent.name,
            self.home_dir.as_deref(),
            redactions,
        );
        redact_secret_option(&format!("{prefix}.id"), &mut agent.id, redactions);
        self.project_uri_or_path_option(&format!("{prefix}.url"), &mut agent.url, true, redactions);
        redact_secret_option(
            &format!("{prefix}.provider"),
            &mut agent.provider,
            redactions,
        );
        redact_secret_option(
            &format!("{prefix}.modelIdentifier"),
            &mut agent.model_identifier,
            redactions,
        );

        for (index, identifier) in agent.identifiers.iter_mut().enumerate() {
            redact_secret_option(
                &format!("{prefix}.identifiers[{index}].value"),
                &mut identifier.value,
                redactions,
            );
        }
    }
}

fn project_agents<'a>(
    _prefix: &str,
    agents: impl Iterator<Item = &'a mut AgentSnapshot>,
    policy: &ProjectionPolicy,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    for (index, agent) in agents.enumerate() {
        policy.project_agent(&format!("attributions[{index}].agent"), agent, redactions);
    }
}

fn project_execution_digest(
    prefix: &str,
    digest: &mut ExecutionDigestSnapshot,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    redact_secret_option(
        &format!("{prefix}.state"),
        &mut digest.state_digest,
        redactions,
    );
    redact_secret_option(
        &format!("{prefix}.semantic"),
        &mut digest.semantic_digest,
        redactions,
    );
    redact_secret_option(
        &format!("{prefix}.dependencies"),
        &mut digest.dependencies_digest,
        redactions,
    );
}

fn project_text_option(
    field: &str,
    value: &mut Option<String>,
    home_dir: Option<&Path>,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    let Some(text) = value.as_deref() else {
        return;
    };

    if has_secret(text) {
        *value = None;
        redactions.push(redaction(field, REDACTION_FULLY_REDACTED));
    } else if contains_home_path(text, home_dir) {
        *value = Some(REDACTED_NAME.to_string());
        redactions.push(redaction(field, REDACTION_NAME_REDACTED));
    }
}

fn project_string_list(
    field: &str,
    values: &mut [String],
    home_dir: Option<&Path>,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    for (index, value) in values.iter_mut().enumerate() {
        redact_secret_or_home_string(&format!("{field}[{index}]"), value, home_dir, redactions);
    }
}

fn redact_secret_string(field: &str, value: &mut String, redactions: &mut Vec<RedactionSnapshot>) {
    if has_secret(value) {
        value.clear();
        value.push_str(REDACTED_NAME);
        redactions.push(redaction(field, REDACTION_NAME_REDACTED));
    }
}

fn redact_secret_or_home_string(
    field: &str,
    value: &mut String,
    home_dir: Option<&Path>,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    if has_secret(value) || contains_home_path(value, home_dir) {
        value.clear();
        value.push_str(REDACTED_NAME);
        redactions.push(redaction(field, REDACTION_NAME_REDACTED));
    }
}

fn project_metadata(
    field: &str,
    value: &mut Value,
    home_dir: Option<&Path>,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    match value {
        Value::String(string) => {
            if has_secret(string) || contains_home_path(string, home_dir) {
                *value = Value::String(REDACTED_NAME.to_string());
                redactions.push(redaction(field, REDACTION_NAME_REDACTED));
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter_mut().enumerate() {
                project_metadata(&format!("{field}[{index}]"), value, home_dir, redactions);
            }
        }
        Value::Object(map) => project_metadata_object(field, map, home_dir, redactions),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn project_metadata_object(
    field: &str,
    map: &mut Map<String, Value>,
    home_dir: Option<&Path>,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    for (key, value) in map.iter_mut() {
        let child = format!("{field}.{key}");
        if is_secret_key(key) {
            *value = Value::String(REDACTED_NAME.to_string());
            redactions.push(redaction(&child, REDACTION_NAME_REDACTED));
        } else {
            project_metadata(&child, value, home_dir, redactions);
        }
    }
}

fn redact_name_option(
    field: &str,
    value: &mut Option<String>,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    if value.is_some() {
        *value = Some(REDACTED_NAME.to_string());
        redactions.push(redaction(field, REDACTION_NAME_REDACTED));
    }
}

fn redact_secret_option(
    field: &str,
    value: &mut Option<String>,
    redactions: &mut Vec<RedactionSnapshot>,
) {
    if has_secret_option(value.as_deref()) {
        *value = None;
        redactions.push(redaction(field, REDACTION_FULLY_REDACTED));
    }
}

fn has_secret_option(value: Option<&str>) -> bool {
    value.is_some_and(has_secret)
}

fn contains_home_path(value: &str, home_dir: Option<&Path>) -> bool {
    let Some(home_dir) = home_dir else {
        return false;
    };
    let home = home_dir.to_string_lossy();
    !home.is_empty() && value.contains(home.as_ref())
}

fn has_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("bearer ")
        || lower.contains("authorization:")
        || lower.contains("authorization=")
        || lower.contains("access_token=")
        || lower.contains("refresh_token=")
        || lower.contains("api_key=")
        || lower.contains("apikey=")
        || lower.contains("token=")
        || lower.contains("secret=")
        || lower.contains("secret:")
        || lower.contains("password=")
        || lower.contains("password:")
        || lower.contains("sig=")
        || lower.contains("x-amz-signature=")
}

fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower == "authorization"
}

fn public_url(value: &str) -> bool {
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };

    if !matches!(url.scheme(), "https" | "http") {
        return false;
    }
    if !url.username().is_empty() || url.password().is_some() || url.query().is_some() {
        return false;
    }

    true
}

fn public_web_url(value: &str) -> bool {
    if !public_url(value) {
        return false;
    }

    let Ok(url) = url::Url::parse(value) else {
        return false;
    };

    url.host_str().is_some_and(|host| !private_url_host(host))
}

fn private_url_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    let labels = host.rsplit('.').collect::<Vec<_>>();
    if labels.len() <= 1
        || labels.first().is_some_and(|label| {
            matches!(
                *label,
                "localhost"
                    | "local"
                    | "internal"
                    | "corp"
                    | "lan"
                    | "home"
                    | "intranet"
                    | "test"
                    | "invalid"
                    | "example"
            )
        })
    {
        return true;
    }

    host.parse::<IpAddr>().is_ok_and(private_ip)
}

fn private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_broadcast()
                || ip.is_documentation()
        }
        IpAddr::V6(ip) => {
            let first = ip.segments()[0];
            ip.is_loopback()
                || ip.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
        }
    }
}

fn is_probable_url(value: &str) -> bool {
    value.contains("://") || value.starts_with("git@")
}

/// Whether `value` is a URL on a well-known public source-host domain.
///
/// Public profile redaction keeps URLs from these hosts because the URL is
/// itself already public information — stripping it loses provenance value
/// without protecting anything.
fn is_public_hosting_url(value: &str) -> bool {
    if !public_url(value) {
        return false;
    }

    let Ok(url) = url::Url::parse(value) else {
        return false;
    };

    let Some(host) = url.host_str() else {
        return false;
    };

    let host = host.trim_end_matches('.').to_ascii_lowercase();
    PUBLIC_HOSTING_DOMAINS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

/// Public-by-default source-hosting domains.
///
/// Repositories on these hosts are public-or-paywalled-public infrastructure
/// where disclosing the URL itself does not leak private information; private
/// repositories on the same host are protected by access control on the host
/// rather than by stripping URL strings from manifests.
const PUBLIC_HOSTING_DOMAINS: &[&str] = &[
    "github.com",
    "gitlab.com",
    "bitbucket.org",
    "codeberg.org",
    "gitee.com",
    "git.sr.ht",
    "sourcehut.org",
];

fn redaction(field: impl Into<String>, reason: impl Into<String>) -> RedactionSnapshot {
    RedactionSnapshot {
        field: Some(field.into()),
        reason: Some(reason.into()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::snapshot::{
        ActivitySnapshot, AttributionSnapshot, DisclosureAssessmentSnapshot,
        ExecutionMessageSnapshot, ExecutionSnapshot, FileDigestSnapshot, PrivacySnapshot,
        ProducerSnapshot, ReproducibilitySnapshot,
    };

    #[test]
    fn public_profile_redacts_private_paths_urls_and_secret_ids() {
        let workspace = PathBuf::from("/home/alice/work/project");
        let home = PathBuf::from("/home/alice");
        let policy =
            ProjectionPolicy::with_roots(CredentialProfile::Public, Some(workspace), Some(home));

        let snapshot = ProvenanceSnapshot {
            source: Some(SourceSnapshot {
                repository: Some("git@github.com:private/repo.git".to_string()),
                path: Some("/home/alice/secret/project/article.smd".to_string()),
                dirty: Some(true),
                patch_digest: Some("sha256:patch".to_string()),
                ..Default::default()
            }),
            environment: Some(EnvironmentSnapshot {
                lockfiles: vec![FileDigestSnapshot {
                    path: Some("/home/alice/work/project/Cargo.lock".to_string()),
                    digest: Some("sha256:lock".to_string()),
                }],
                ..Default::default()
            }),
            attributions: vec![AttributionSnapshot {
                agent: AgentSnapshot {
                    id: Some("Bearer abc123".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };

        let projected = policy.project_snapshot(snapshot);

        let source = projected.source.expect("source");
        assert!(source.repository.is_none());
        assert!(source.path.is_none());
        assert!(source.patch_digest.is_none());

        let lockfile = &projected.environment.expect("environment").lockfiles[0];
        assert_eq!(lockfile.path.as_deref(), Some("Cargo.lock"));
        assert_eq!(lockfile.digest.as_deref(), Some("sha256:lock"));

        let reasons: Vec<_> = projected
            .privacy
            .expect("privacy")
            .redactions
            .into_iter()
            .filter_map(|redaction| redaction.reason)
            .collect();
        assert!(reasons.contains(&REDACTION_URI_OMITTED.to_string()));
        assert!(reasons.contains(&REDACTION_FULLY_REDACTED.to_string()));
    }

    #[test]
    fn public_profile_omits_repository_urls_conservatively() {
        let policy = ProjectionPolicy::with_roots(CredentialProfile::Public, None, None);
        let snapshot = ProvenanceSnapshot {
            source: Some(SourceSnapshot {
                repository: Some("https://git.example.internal/private/repo".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let projected = policy.project_snapshot(snapshot);
        assert!(projected.source.expect("source").repository.is_none());
    }

    #[test]
    fn public_profile_redacts_absolute_paths_and_private_agent_urls() {
        let policy = ProjectionPolicy::with_roots(
            CredentialProfile::Public,
            Some(PathBuf::from("/workspace/project")),
            None,
        );
        let snapshot = ProvenanceSnapshot {
            executed_node: Some(DocumentSnapshot {
                node_type: "CodeChunk".to_string(),
                node_path: Some("/srv/private/project/article.smd#chunk-1".to_string()),
                content_url: Some("http://localhost:8000/output.png".to_string()),
                ..Default::default()
            }),
            attributions: vec![AttributionSnapshot {
                agent: AgentSnapshot {
                    url: Some("http://10.0.0.5/agent/profile".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };

        let projected = policy.project_snapshot(snapshot);
        let node = projected.executed_node.expect("executed node");
        assert!(node.node_path.is_none());
        assert!(node.content_url.is_none());
        assert!(projected.attributions[0].agent.url.is_none());
    }

    #[test]
    fn public_profile_keeps_repository_urls_on_known_public_hosts() {
        let policy = ProjectionPolicy::with_roots(CredentialProfile::Public, None, None);
        let snapshot = ProvenanceSnapshot {
            source: Some(SourceSnapshot {
                repository: Some("https://github.com/stencila/stencila".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let projected = policy.project_snapshot(snapshot);
        assert_eq!(
            projected.source.expect("source").repository.as_deref(),
            Some("https://github.com/stencila/stencila"),
            "URLs on well-known public hosting domains should pass through under the Public profile"
        );
    }

    #[test]
    fn public_profile_strips_repository_urls_with_secret_query_parameters() {
        let policy = ProjectionPolicy::with_roots(CredentialProfile::Public, None, None);
        let snapshot = ProvenanceSnapshot {
            source: Some(SourceSnapshot {
                repository: Some("https://github.com/stencila/stencila?token=abc123".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let projected = policy.project_snapshot(snapshot);
        assert!(projected.source.expect("source").repository.is_none());
    }

    #[test]
    fn public_profile_redacts_secret_and_home_path_fields_across_snapshot() {
        let home = PathBuf::from("/home/alice");
        let policy = ProjectionPolicy::with_roots(CredentialProfile::Public, None, Some(home));
        let snapshot = ProvenanceSnapshot {
            root_node: DocumentSnapshot {
                node_type: "Article".to_string(),
                title: Some("Draft at /home/alice/private/article.smd".to_string()),
                ..Default::default()
            },
            executed_node: Some(DocumentSnapshot {
                node_type: "CodeChunk".to_string(),
                node_path: Some("/home/alice/private/article.smd#chunk-1".to_string()),
                title: Some("Chunk at /home/alice/private/article.smd".to_string()),
                ..Default::default()
            }),
            activity: Some(ActivitySnapshot {
                name: Some("render with Authorization: Bearer abc123".to_string()),
                used_node_ids: vec!["/home/alice/private/customers.csv".to_string()],
                ..Default::default()
            }),
            producer: Some(ProducerSnapshot {
                renderer: Some("renderer password: abc123".to_string()),
                ..Default::default()
            }),
            execution: Some(ExecutionSnapshot {
                messages: vec![ExecutionMessageSnapshot {
                    message: Some("failed reading /home/alice/private/customers.csv".to_string()),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            attributions: vec![AttributionSnapshot {
                agent: AgentSnapshot {
                    name: Some("/home/alice/private/agent-profile.json".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }],
            reproducibility: Some(ReproducibilitySnapshot {
                comparison: Some(json!({
                    "token": "abc123",
                    "message": "see /home/alice/private/result.json"
                })),
                ..Default::default()
            }),
            ..Default::default()
        };

        let projected = policy.project_snapshot(snapshot);
        assert!(
            projected
                .executed_node
                .as_ref()
                .expect("executed node")
                .node_path
                .is_none()
        );
        assert_eq!(projected.root_node.title.as_deref(), Some(REDACTED_NAME));
        assert_eq!(
            projected
                .executed_node
                .as_ref()
                .expect("executed node")
                .title
                .as_deref(),
            Some(REDACTED_NAME)
        );
        assert_eq!(
            projected.activity.as_ref().expect("activity").used_node_ids[0],
            REDACTED_NAME
        );
        assert_eq!(
            projected.execution.as_ref().expect("execution").messages[0]
                .message
                .as_deref(),
            Some(REDACTED_NAME)
        );
        assert!(
            projected
                .producer
                .as_ref()
                .expect("producer")
                .renderer
                .is_none()
        );

        let json = serde_json::to_string(&projected).expect("serialize");
        assert!(!json.contains("/home/alice"));
        assert!(!json.contains("Bearer abc123"));
        assert!(!json.contains("password: abc123"));
    }

    #[test]
    fn projection_preserves_existing_privacy_assessments() {
        let policy = ProjectionPolicy::with_roots(CredentialProfile::Public, None, None);
        let snapshot = ProvenanceSnapshot {
            privacy: Some(PrivacySnapshot {
                personal_data: DisclosureAssessmentSnapshot {
                    status: "assessed".to_string(),
                    assessed_at: Some("2026-05-09T00:00:00Z".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        let projected = policy.project_snapshot(snapshot);
        let privacy = projected.privacy.expect("privacy");
        assert_eq!(privacy.personal_data.status, "assessed");
        assert_eq!(
            privacy.personal_data.assessed_at.as_deref(),
            Some("2026-05-09T00:00:00Z")
        );
        assert_eq!(
            privacy.personal_data.policy.as_deref(),
            Some("org.stencila.credentials.public.v1")
        );
    }

    #[test]
    fn assertion_size_is_bounded() {
        let policy = ProjectionPolicy::for_profile(CredentialProfile::Public);
        let mut assertion = ProvenanceAssertion::new_v1("text/plain", "sha256:abc");
        assertion.root_node.title = Some("x".repeat(ASSERTION_HARD_SIZE_LIMIT));

        assert!(policy.validate_assertion_size(&assertion).is_err());
    }
}
