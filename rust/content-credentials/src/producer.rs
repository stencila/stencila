//! Sign Stencila assets with a C2PA manifest carrying the
//! `org.stencila.provenance` assertion.

use std::{
    fs::{self, File, Permissions},
    io::{self, Cursor, Write},
    path::{Path, PathBuf},
};

use c2pa::{BoxedSigner, Builder, Context, HashRange, Ingredient, Reader, Relationship};
use serde::Serialize;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

use crate::{
    assertion::asset_kind_for_media_type,
    error::{Error, Result},
    media, pdf,
    policy::{CredentialProfile, ProjectionPolicy},
    schema::{PROVENANCE_LABEL, ProvenanceAssertion},
    signer::CredentialSignerConfig,
    snapshot::{
        AssetSnapshot, IngredientRelationship, IngredientSnapshot, IngredientThumbnailSnapshot,
        ProvenanceSnapshot,
    },
    thumbnails::{self, StaticThumbnail},
};

/// Maximum image size, in bytes, that is embedded as-is as a `c2pa.thumbnail.claim`.
///
/// Larger images are skipped so manifests stay compact.
const MAX_THUMBNAIL_BYTES: u64 = 256 * 1024;

/// Whether a manifest is embedded in the asset bytes or written to a sidecar.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ManifestKind {
    Embedded,
    Sidecar,
}

impl ManifestKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::Sidecar => "sidecar",
        }
    }
}

/// Inputs for signing an exported asset.
#[derive(Debug, Clone, Default)]
pub struct SignAssetRequest {
    pub input_path: PathBuf,

    /// Optional media type override.
    ///
    /// When absent, the media type is inferred from [`input_path`](Self::input_path).
    /// Use this for Stencila-owned formats whose extension is not reliably
    /// represented by generic MIME databases.
    pub media_type: Option<String>,

    /// Where the signed asset should be written.
    ///
    /// `None` means in-place for embedded manifests, or sidecar-next-to-input
    /// for non-embeddable formats.
    ///
    /// When supplied, the output extension must resolve to the same media type
    /// as the input so future verification uses the correct C2PA handler.
    pub output_path: Option<PathBuf>,

    /// Optional human-readable title surfaced in the C2PA manifest.
    pub title: Option<String>,

    /// Optional Stencila provenance facts to embed in `org.stencila.provenance`.
    pub provenance: Option<ProvenanceSnapshot>,

    /// Privacy projection profile applied before signing.
    pub credential_profile: CredentialProfile,
}

/// Result of a successful signing operation.
#[derive(Debug, Clone)]
pub struct SignedAsset {
    pub asset_path: PathBuf,
    pub manifest_kind: ManifestKind,
    pub manifest_id: Option<String>,
    pub sidecar_path: Option<PathBuf>,
    pub assertion_label: &'static str,
    pub assertion_schema: &'static str,
    pub source_digest: String,
    pub signed_asset_digest: String,
    pub media_type: String,
    pub credential_profile: CredentialProfile,
    pub warnings: Vec<String>,
}

/// High-level credential producer.
pub struct CredentialProducer {
    signer: CredentialSignerConfig,
}

impl CredentialProducer {
    #[must_use]
    pub fn new(signer: CredentialSignerConfig) -> Self {
        Self { signer }
    }

    /// Sign an asset and emit either an embedded manifest or a sidecar.
    ///
    /// # Errors
    ///
    /// Returns an error if the input asset does not exist, its media type or
    /// digest cannot be determined, the signer cannot be created, the c2pa SDK
    /// cannot sign the asset, file IO fails, or the blocking signing task fails.
    pub async fn sign_exported_asset(&self, request: SignAssetRequest) -> Result<SignedAsset> {
        let SignAssetRequest {
            input_path,
            media_type,
            output_path,
            title,
            provenance,
            credential_profile,
        } = request;

        if !input_path.exists() {
            return Err(Error::InputNotFound(input_path));
        }

        let media_type = media_type.map_or_else(|| media::guess_media_type(&input_path), Ok)?;
        let embed = media::embed_supported(&media_type);
        if let Some(output_path) = output_path.as_deref() {
            if !embed && media::sidecar_path(output_path) == output_path {
                return Err(Error::OutputSidecarConflict(output_path.to_path_buf()));
            }
            validate_output_media_type(&input_path, &media_type, output_path)?;
        }

        let source_digest = media::sha256_file(&input_path)?;
        let signer = self.signer.clone();
        let fallback_title = input_path
            .file_name()
            .and_then(|n| n.to_str().map(std::string::ToString::to_string));

        let policy = ProjectionPolicy::for_profile(credential_profile);
        let (assertion, ingredients, title) = prepare_signing_claim(
            provenance,
            &media_type,
            &source_digest,
            title,
            fallback_title,
            &policy,
        );
        policy.validate_assertion_size(&assertion)?;

        let media_for_task = media_type.clone();
        let source_digest_for_result = source_digest.clone();
        let media_type_for_result = media_type.clone();

        // c2pa's signer is sync; run on a blocking thread.
        let result = tokio::task::spawn_blocking(move || {
            if embed {
                sign_embedded(
                    &input_path,
                    output_path.as_deref(),
                    &media_for_task,
                    &title,
                    &assertion,
                    &ingredients,
                    &signer,
                )
            } else {
                sign_sidecar(
                    &input_path,
                    output_path.as_deref(),
                    &media_for_task,
                    &title,
                    &assertion,
                    &ingredients,
                    &signer,
                )
            }
        })
        .await??;

        let signed_asset_digest = media::sha256_file(&result.0)?;
        let (manifest_id, warnings) =
            read_signed_manifest_id(&result.0, result.1.as_deref(), &media_type_for_result);

        Ok(SignedAsset {
            asset_path: result.0,
            manifest_kind: if embed {
                ManifestKind::Embedded
            } else {
                ManifestKind::Sidecar
            },
            manifest_id,
            sidecar_path: result.1,
            assertion_label: PROVENANCE_LABEL,
            assertion_schema: crate::schema::PROVENANCE_SCHEMA,
            source_digest: source_digest_for_result,
            signed_asset_digest,
            media_type: media_type_for_result,
            credential_profile,
            warnings,
        })
    }
}

fn prepare_signing_claim(
    provenance: Option<ProvenanceSnapshot>,
    media_type: &str,
    source_digest: &str,
    title: Option<String>,
    fallback_title: Option<String>,
    policy: &ProjectionPolicy,
) -> (ProvenanceAssertion, Vec<IngredientSnapshot>, String) {
    let snapshot = provenance.map_or_else(
        || {
            ProvenanceSnapshot::for_asset(AssetSnapshot::new(
                asset_kind_for_media_type(media_type),
                media_type,
                source_digest,
            ))
        },
        |mut snapshot| {
            snapshot.asset = normalize_asset_snapshot(snapshot.asset, media_type, source_digest);
            snapshot
        },
    );
    let mut snapshot = policy.project_snapshot(snapshot);
    let title = title
        .or_else(|| manifest_title_from_snapshot(&snapshot))
        .or(fallback_title)
        .unwrap_or_else(|| "asset".to_string());
    let ingredients = std::mem::take(&mut snapshot.ingredients);

    (
        ProvenanceAssertion::from_snapshot(snapshot),
        ingredients,
        title,
    )
}

fn manifest_title_from_snapshot(snapshot: &ProvenanceSnapshot) -> Option<String> {
    let asset_title = clean_title(snapshot.asset.title.as_deref());
    let root_title = clean_title(snapshot.root_node.title.as_deref());

    if snapshot.asset.role.as_deref() == Some("document-export") {
        root_title.or(asset_title)
    } else {
        asset_title.or(root_title)
    }
}

fn clean_title(title: Option<&str>) -> Option<String> {
    let title = title?.trim();
    (!title.is_empty()).then(|| title.to_string())
}

fn read_signed_manifest_id(
    asset_path: &Path,
    sidecar_path: Option<&Path>,
    media_type: &str,
) -> (Option<String>, Vec<String>) {
    match read_signed_reader(asset_path, sidecar_path, media_type) {
        Ok(reader) => {
            let manifest_id = reader.active_label().map(ToString::to_string).or_else(|| {
                reader
                    .active_manifest()
                    .map(|manifest| manifest.instance_id().to_string())
            });
            if manifest_id.is_some() {
                (manifest_id, Vec::new())
            } else {
                (
                    None,
                    vec!["signed asset did not expose an active manifest id".to_string()],
                )
            }
        }
        Err(error) => (
            None,
            vec![format!(
                "signed asset could not be re-read for manifest id: {error}"
            )],
        ),
    }
}

fn read_signed_reader(
    asset_path: &Path,
    sidecar_path: Option<&Path>,
    media_type: &str,
) -> Result<Reader> {
    let reader = Reader::from_context(Context::new());
    if let Some(sidecar_path) = sidecar_path {
        let manifest_bytes = fs::read(sidecar_path)?;
        let mut asset = File::open(asset_path)?;
        reader
            .with_manifest_data_and_stream(&manifest_bytes, media_type, &mut asset)
            .map_err(Error::C2pa)
    } else {
        let mut asset = File::open(asset_path)?;
        reader
            .with_stream(media_type, &mut asset)
            .map_err(Error::C2pa)
    }
}

fn normalize_asset_snapshot(
    mut asset: AssetSnapshot,
    media_type: &str,
    source_digest: &str,
) -> AssetSnapshot {
    asset.media_type = media_type.to_string();
    asset.content_digest = source_digest.to_string();
    if asset.kind.is_empty() || asset.kind == "asset" {
        asset.kind = asset_kind_for_media_type(media_type).to_string();
    }
    asset
}

fn validate_output_media_type(
    input_path: &Path,
    input_media_type: &str,
    output_path: &Path,
) -> Result<()> {
    let output_media_type = media::guess_media_type(output_path)?;
    if output_media_type != input_media_type {
        return Err(Error::OutputMediaTypeMismatch {
            input_path: input_path.to_path_buf(),
            input_media_type: input_media_type.to_string(),
            output_path: output_path.to_path_buf(),
            output_media_type,
        });
    }

    Ok(())
}

/// Sign with an embedded manifest. Returns (`asset_path`, None).
fn sign_embedded(
    input_path: &Path,
    output_path: Option<&Path>,
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
    signer_config: &CredentialSignerConfig,
) -> Result<(PathBuf, Option<PathBuf>)> {
    if media_type == "application/pdf" {
        return sign_embedded_pdf(
            input_path,
            output_path,
            media_type,
            title,
            assertion,
            ingredients,
            signer_config,
        );
    }

    let dest_path = output_path.map_or_else(|| input_path.to_path_buf(), Path::to_path_buf);

    let parent = dest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    fs::create_dir_all(&parent)?;

    let mut builder = build_builder(media_type, title, assertion, ingredients, Some(input_path))?;
    builder.set_no_embed(false);

    let signer = signer_config.create_signer()?;
    let permissions = fs::metadata(input_path)?.permissions();

    let mut tmp = NamedTempFile::new_in(&parent)?;
    {
        let mut source = File::open(input_path)?;
        let file = tmp.as_file_mut();
        builder.sign(signer.as_ref(), media_type, &mut source, file)?;
        file.flush()?;
    }
    persist_with_permissions(tmp, &dest_path, &permissions)?;

    Ok((dest_path, None))
}

/// Sign a PDF by embedding a fixed-size C2PA placeholder, hashing the resulting
/// PDF with that byte range excluded, then patching the signed manifest into
/// the same range.
fn sign_embedded_pdf(
    input_path: &Path,
    output_path: Option<&Path>,
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
    signer_config: &CredentialSignerConfig,
) -> Result<(PathBuf, Option<PathBuf>)> {
    let dest_path = output_path.map_or_else(|| input_path.to_path_buf(), Path::to_path_buf);

    let parent = dest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    fs::create_dir_all(&parent)?;

    let signer = signer_config.create_signer()?;
    let permissions = fs::metadata(input_path)?.permissions();
    let mut builder = build_builder_with_signer(
        media_type,
        title,
        assertion,
        ingredients,
        Some(input_path),
        signer,
    )?;
    builder.set_no_embed(false);

    let placeholder = builder.placeholder(media_type)?;
    if placeholder.is_empty() {
        return Err(Error::other("PDF C2PA placeholder was empty"));
    }

    let (mut pdf_bytes, manifest_range) = pdf::with_manifest_bytes(input_path, &placeholder)?;
    let mut pdf_stream = Cursor::new(pdf_bytes.as_slice());
    builder.set_data_hash_exclusions(vec![HashRange::new(
        manifest_range.start as u64,
        manifest_range.len() as u64,
    )])?;
    builder.update_hash_from_stream(media_type, &mut pdf_stream)?;

    let signed_manifest = builder.sign_embeddable(media_type)?;
    if signed_manifest.len() != manifest_range.len() {
        return Err(Error::other(format!(
            "signed PDF manifest length {} did not match placeholder length {}",
            signed_manifest.len(),
            manifest_range.len()
        )));
    }

    pdf_bytes[manifest_range].copy_from_slice(&signed_manifest);
    let mut verify_stream = Cursor::new(pdf_bytes.as_slice());
    Reader::from_context(Context::new())
        .with_stream(media_type, &mut verify_stream)
        .map_err(Error::C2pa)?;

    let mut tmp = NamedTempFile::new_in(&parent)?;
    tmp.write_all(&pdf_bytes)?;
    tmp.flush()?;
    persist_with_permissions(tmp, &dest_path, &permissions)?;

    Ok((dest_path, None))
}

/// Sign with a `.c2pa` sidecar next to the asset. Returns (`asset_path`, `Some(sidecar_path)`).
fn sign_sidecar(
    input_path: &Path,
    output_path: Option<&Path>,
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
    signer_config: &CredentialSignerConfig,
) -> Result<(PathBuf, Option<PathBuf>)> {
    let asset_dest = output_path.map_or_else(|| input_path.to_path_buf(), Path::to_path_buf);
    let sidecar_dest = media::sidecar_path(&asset_dest);
    if sidecar_dest == asset_dest {
        return Err(Error::OutputSidecarConflict(asset_dest));
    }

    let parent = asset_dest
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    fs::create_dir_all(&parent)?;

    let mut builder = build_builder(media_type, title, assertion, ingredients, Some(input_path))?;
    builder.set_no_embed(true);

    let signer = signer_config.create_signer()?;
    let permissions = fs::metadata(input_path)?.permissions();

    if media::sidecar_requires_prehashed_manifest(media_type) {
        let request = PrehashedSidecarRequest {
            input_path,
            asset_dest: &asset_dest,
            sidecar_dest: &sidecar_dest,
            media_type,
            title,
            assertion,
            ingredients,
            permissions: &permissions,
        };
        return sign_prehashed_sidecar(&request, signer);
    }

    if !media::could_have_embedded(media_type) {
        let request = PrehashedSidecarRequest {
            input_path,
            asset_dest: &asset_dest,
            sidecar_dest: &sidecar_dest,
            media_type,
            title,
            assertion,
            ingredients,
            permissions: &permissions,
        };
        return sign_unsupported_sidecar(&request, signer);
    }

    // Capture both outputs from c2pa. For supported writable formats the SDK
    // may rewrite the asset stream, for example to strip an existing embedded
    // manifest before computing the hard binding.
    let mut source = File::open(input_path)?;
    let mut sink = Cursor::new(Vec::<u8>::new());

    let manifest_bytes = builder.sign(signer.as_ref(), media_type, &mut source, &mut sink)?;
    let signed_asset_bytes = sink.into_inner();
    drop(source);

    let mut tmp_asset = NamedTempFile::new_in(&parent)?;
    tmp_asset.write_all(&signed_asset_bytes)?;
    tmp_asset.flush()?;
    persist_with_permissions(tmp_asset, &asset_dest, &permissions)?;

    let mut tmp_sidecar = NamedTempFile::new_in(&parent)?;
    tmp_sidecar.write_all(&manifest_bytes)?;
    tmp_sidecar.flush()?;
    persist_with_permissions(tmp_sidecar, &sidecar_dest, &permissions)?;

    Ok((asset_dest, Some(sidecar_dest)))
}

struct PrehashedSidecarRequest<'a> {
    input_path: &'a Path,
    asset_dest: &'a Path,
    sidecar_dest: &'a Path,
    media_type: &'a str,
    title: &'a str,
    assertion: &'a ProvenanceAssertion,
    ingredients: &'a [IngredientSnapshot],
    permissions: &'a Permissions,
}

fn sign_prehashed_sidecar(
    request: &PrehashedSidecarRequest,
    signer: BoxedSigner,
) -> Result<(PathBuf, Option<PathBuf>)> {
    let parent = request
        .asset_dest
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

    let mut builder = build_builder_with_signer(
        request.media_type,
        request.title,
        request.assertion,
        request.ingredients,
        Some(request.input_path),
        signer,
    )?;
    builder.set_no_embed(true);

    let mut source = File::open(request.input_path)?;
    builder.update_hash_from_stream(request.media_type, &mut source)?;
    let manifest_bytes = builder.sign_embeddable(request.media_type)?;

    let mut source = File::open(request.input_path)?;
    let mut tmp_asset = NamedTempFile::new_in(&parent)?;
    io::copy(&mut source, &mut tmp_asset)?;
    tmp_asset.flush()?;
    persist_with_permissions(tmp_asset, request.asset_dest, request.permissions)?;

    let mut tmp_sidecar = NamedTempFile::new_in(&parent)?;
    tmp_sidecar.write_all(&manifest_bytes)?;
    tmp_sidecar.flush()?;
    persist_with_permissions(tmp_sidecar, request.sidecar_dest, request.permissions)?;

    Ok((
        request.asset_dest.to_path_buf(),
        Some(request.sidecar_dest.to_path_buf()),
    ))
}

fn sign_unsupported_sidecar(
    request: &PrehashedSidecarRequest,
    signer: BoxedSigner,
) -> Result<(PathBuf, Option<PathBuf>)> {
    let parent = request
        .asset_dest
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

    let mut builder = build_builder_with_signer(
        request.media_type,
        request.title,
        request.assertion,
        request.ingredients,
        Some(request.input_path),
        signer,
    )?;
    builder.set_no_embed(true);

    let mut source = File::open(request.input_path)?;
    let mut sink = Cursor::new(Vec::<u8>::new());
    let manifest_bytes = builder.save_to_stream(request.media_type, &mut source, &mut sink)?;
    let signed_asset_bytes = sink.into_inner();

    let mut tmp_asset = NamedTempFile::new_in(&parent)?;
    if signed_asset_bytes.is_empty() {
        let mut source = File::open(request.input_path)?;
        io::copy(&mut source, &mut tmp_asset)?;
    } else {
        tmp_asset.write_all(&signed_asset_bytes)?;
    }
    tmp_asset.flush()?;
    persist_with_permissions(tmp_asset, request.asset_dest, request.permissions)?;

    let mut tmp_sidecar = NamedTempFile::new_in(&parent)?;
    tmp_sidecar.write_all(&manifest_bytes)?;
    tmp_sidecar.flush()?;
    persist_with_permissions(tmp_sidecar, request.sidecar_dest, request.permissions)?;

    Ok((
        request.asset_dest.to_path_buf(),
        Some(request.sidecar_dest.to_path_buf()),
    ))
}

fn persist_with_permissions(
    tmp: NamedTempFile,
    dest_path: &Path,
    permissions: &Permissions,
) -> Result<()> {
    tmp.persist(dest_path).map_err(|err| Error::Io(err.error))?;
    fs::set_permissions(dest_path, permissions.clone())?;
    Ok(())
}

fn build_builder(
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
    thumbnail_source: Option<&Path>,
) -> Result<Builder> {
    build_builder_with_context(
        Context::new(),
        media_type,
        title,
        assertion,
        ingredients,
        thumbnail_source,
    )
}

fn build_builder_with_signer(
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
    thumbnail_source: Option<&Path>,
    signer: BoxedSigner,
) -> Result<Builder> {
    build_builder_with_context(
        Context::new().with_signer(signer),
        media_type,
        title,
        assertion,
        ingredients,
        thumbnail_source,
    )
}

fn build_builder_with_context(
    context: Context,
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
    thumbnail_source: Option<&Path>,
) -> Result<Builder> {
    let definition = json!({
        "claim_generator_info": [{
            "name": "Stencila",
            "version": stencila_version::STENCILA_VERSION,
        }],
        "title": title,
        "format": media_type,
        "assertions": standard_assertions(assertion, media_type, title, ingredients)
    });

    let mut builder = Builder::from_context(context).with_definition(definition.to_string())?;
    builder.add_assertion(PROVENANCE_LABEL, assertion)?;

    for (index, ingredient) in ingredients.iter().enumerate() {
        attach_ingredient(&mut builder, ingredient, index)?;
    }

    apply_claim_thumbnail(&mut builder, media_type, title, assertion, thumbnail_source);

    Ok(builder)
}

fn apply_claim_thumbnail(
    builder: &mut Builder,
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
    asset_path: Option<&Path>,
) {
    if let Some(thumbnail_format) = thumbnail_format(media_type) {
        if let Some(path) = asset_path
            && let Ok(metadata) = fs::metadata(path)
            && metadata.len() <= MAX_THUMBNAIL_BYTES
            && let Ok(mut file) = File::open(path)
        {
            // Best-effort: failures here are not fatal because the thumbnail is
            // purely informational metadata.
            let _ = builder.set_thumbnail(thumbnail_format, &mut file);
        }
        return;
    }

    apply_static_claim_thumbnail(
        builder,
        thumbnails::claim_for_assertion_with_hints(assertion, Some(title), Some(media_type)),
    );
}

fn apply_static_claim_thumbnail(builder: &mut Builder, thumbnail: StaticThumbnail) {
    if thumbnail.bytes.is_empty() || thumbnail.bytes.len() as u64 > MAX_THUMBNAIL_BYTES {
        return;
    }

    let mut bytes = Cursor::new(thumbnail.bytes);
    // Best-effort metadata: the signed asset and Stencila assertion are still
    // valid if a viewer-facing thumbnail cannot be attached.
    let _ = builder.set_thumbnail(thumbnail.media_type, &mut bytes);
}

fn standard_assertions(
    assertion: &ProvenanceAssertion,
    media_type: &str,
    title: &str,
    ingredients: &[IngredientSnapshot],
) -> Vec<Value> {
    let mut assertions = vec![
        actions_assertion(assertion, ingredients),
        metadata_assertion(assertion, media_type, title),
        asset_type_assertion(assertion),
    ];

    if let Some(ai_disclosure) = ai_disclosure_assertion(assertion) {
        assertions.push(ai_disclosure);
    }

    assertions
}

fn actions_assertion(assertion: &ProvenanceAssertion, ingredients: &[IngredientSnapshot]) -> Value {
    // The C2PA spec requires the first action to be `c2pa.created` or
    // `c2pa.opened`. When a caller supplies a single `parentOf` ingredient, it
    // is a direct derivation parent and must be referenced by `c2pa.opened`.
    // Otherwise Stencila exports materialise new bytes directly, so
    // `c2pa.created` remains the first action.
    let opened = opened_action(assertion, ingredients);
    let include_parent_in_created = opened.is_none();
    let mut created = json!({
        "action": "c2pa.created",
        "description": action_description(assertion),
        "softwareAgent": software_agent_value(assertion),
        "parameters": action_parameters(assertion, ingredients, include_parent_in_created),
    });

    if let Some(when) = action_timestamp(assertion) {
        created["when"] = json!(when);
    }

    if let Some(uri) = digital_source_type(assertion) {
        created["digitalSourceType"] = json!(uri);
    }

    let mut actions = Vec::new();
    if let Some(opened) = opened {
        actions.push(opened);
        if let Some(executed) = executed_action(assertion, ingredients) {
            actions.push(executed);
        }
        actions.push(created);
    } else {
        actions.push(created);
        if let Some(executed) = executed_action(assertion, ingredients) {
            actions.push(executed);
        }
    }
    actions.extend(placed_actions(assertion, ingredients));

    json!({
        "label": "c2pa.actions.v2",
        "data": {
            "actions": actions,
        }
    })
}

fn opened_action(
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
) -> Option<Value> {
    let mut parents = ingredients
        .iter()
        .enumerate()
        .filter(|(_, ingredient)| ingredient.relationship == IngredientRelationship::ParentOf);
    let (index, ingredient) = parents.next()?;
    if parents.next().is_some() {
        return None;
    }

    let mut action = json!({
        "action": "c2pa.opened",
        "description": ingredient
            .title
            .as_deref()
            .map_or_else(|| "Open parent asset".to_string(), |title| format!("Open {title}")),
        "softwareAgent": software_agent_value(assertion),
        "parameters": {
            "ingredientIds": [ingredient_label(ingredient, index)]
        }
    });

    if let Some(when) = action_timestamp(assertion) {
        action["when"] = json!(when);
    }

    Some(action)
}

fn executed_action(
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
) -> Option<Value> {
    let execution = assertion.execution.as_ref()?;
    let node = assertion.executed_node.as_ref()?;

    let mut parameters = json!({});
    parameters["org.stencila.nodeType"] = json!(node.node_type);
    if let Some(node_id) = &node.node_id {
        parameters["org.stencila.nodeId"] = json!(node_id);
    }
    if let Some(persistent_id) = &node.persistent_id {
        parameters["org.stencila.persistentId"] = json!(persistent_id);
    }
    if let Some(language) = &node.programming_language {
        parameters["org.stencila.programmingLanguage"] = json!(language);
    }

    if let Ok(value) = serde_json::to_value(execution) {
        parameters["org.stencila.execution"] = value;
    }

    let input_ingredient_ids: Vec<String> = ingredients
        .iter()
        .enumerate()
        .filter(|(_, ingredient)| ingredient.relationship == IngredientRelationship::InputTo)
        .map(|(index, ingredient)| ingredient_label(ingredient, index))
        .collect();
    if !input_ingredient_ids.is_empty() {
        parameters["ingredientIds"] = json!(input_ingredient_ids);
    }

    let mut action = json!({
        "action": "org.stencila.executed",
        "description": format!("Execute {}", node.node_type),
        "softwareAgent": software_agent_value(assertion),
        "parameters": parameters,
    });

    if let Some(when) = execution
        .ended_at
        .clone()
        .or_else(|| action_timestamp(assertion))
    {
        action["when"] = json!(when);
    }

    Some(action)
}

/// IPTC `DigitalSourceType` URI for the action, when one can be inferred from
/// the asset role. See <https://cv.iptc.org/newscodes/digitalsourcetype/> for
/// the controlled vocabulary; the URIs are stable identifiers in C2PA.
fn digital_source_type(assertion: &ProvenanceAssertion) -> Option<&'static str> {
    match assertion.asset.role.as_deref() {
        Some("computational-output" | "figure" | "table-image") => {
            Some("http://cv.iptc.org/newscodes/digitalsourcetype/dataDrivenMedia")
        }
        Some("document-export") => {
            Some("http://cv.iptc.org/newscodes/digitalsourcetype/digitalCreation")
        }
        _ => None,
    }
}

fn metadata_assertion(assertion: &ProvenanceAssertion, media_type: &str, title: &str) -> Value {
    let label = assertion
        .asset
        .title
        .as_deref()
        .or(assertion.asset.label.as_deref())
        .unwrap_or(title);

    // The C2PA spec defines a closed allow-list of fields permitted under
    // `c2pa.metadata` (see C2PA spec metadata_annex). Anything outside that
    // list is rejected with `assertion.metadata.disallowed`. Human-facing
    // document titles are carried by the manifest title and Stencila provenance
    // assertion instead because `dc:title` is not permitted here.
    let mut data = json!({
        "@context": {
            "xmp": "http://ns.adobe.com/xap/1.0/",
            "dc": "http://purl.org/dc/elements/1.1/",
        },
        "xmp:CreatorTool": producer_software_agent(assertion),
        "xmp:Label": label,
        "dc:format": media_type,
    });

    if let Some(dcmi_type) = dcmi_type_for(assertion) {
        data["dc:type"] = json!(dcmi_type);
    }

    if let Some(when) = action_timestamp(assertion) {
        data["xmp:CreateDate"] = json!(when);
    }

    json!({
        "label": "c2pa.metadata",
        "kind": "Json",
        "data": data,
    })
}

/// Map the asset's broad type to a DCMI Type vocabulary term.
///
/// The DCMI Type vocabulary at <https://www.dublincore.org/specifications/dublin-core/dcmi-terms/#section-7>
/// is a closed list. Returning `None` here is preferable to emitting an
/// out-of-vocabulary string under `dc:type`.
fn dcmi_type_for(assertion: &ProvenanceAssertion) -> Option<&'static str> {
    match assertion.asset.asset_type.as_str() {
        "image" | "figure" => Some("StillImage"),
        "document" => Some("Text"),
        "dataset" | "table" => Some("Dataset"),
        _ => None,
    }
}

fn asset_type_assertion(assertion: &ProvenanceAssertion) -> Value {
    // The C2PA asset-type assertion has no `.v2` form: a previous `.v2` suffix
    // here was silently normalised away by the SDK before being stored. Use the
    // spec label so the wire form matches the code.
    json!({
        "label": "c2pa.asset-type",
        "data": {
            "types": [{
                "type": standard_asset_type(assertion),
                "dc:format": assertion.asset.media_type,
            }]
        }
    })
}

/// Standard `c2pa.ai-disclosure` assertion.
///
/// Emitted only when the snapshot supplies a fully-formed `standard_assertion`
/// JSON body. The producer does not synthesise an AI disclosure assertion from
/// the snapshot's other fields because the C2PA AI disclosure schema is
/// detailed enough that producing a partial body risks asserting more than the
/// caller has actually verified. The companion Stencila assertion's
/// `aiDisclosure` record continues to carry Stencila-specific identifiers in
/// either case.
fn ai_disclosure_assertion(assertion: &ProvenanceAssertion) -> Option<Value> {
    let disclosure = assertion.ai_disclosure.as_ref()?;
    let body_json = disclosure.standard_assertion.as_deref()?.trim();
    if body_json.is_empty() {
        return None;
    }
    let body: Value = serde_json::from_str(body_json).ok()?;
    Some(json!({
        "label": "c2pa.ai-disclosure",
        "kind": "Json",
        "data": body,
    }))
}

/// Maps a media type to a thumbnail format the c2pa builder accepts, or
/// `None` when no thumbnail should be embedded.
fn thumbnail_format(media_type: &str) -> Option<&'static str> {
    match media_type {
        "image/png" => Some("image/png"),
        "image/jpeg" | "image/jpg" => Some("image/jpeg"),
        "image/gif" => Some("image/gif"),
        "image/svg+xml" => Some("image/svg+xml"),
        "image/webp" => Some("image/webp"),
        _ => None,
    }
}

/// Add an ingredient to the builder, linking the child manifest when one
/// is available so that verifiers can chain provenance across assets.
fn attach_ingredient(
    builder: &mut Builder,
    snapshot: &IngredientSnapshot,
    index: usize,
) -> Result<()> {
    if let Some(manifest_source) = snapshot.manifest_source.as_deref() {
        match attach_ingredient_with_manifest(builder, snapshot, index, manifest_source) {
            Ok(()) => return Ok(()),
            Err(error) => {
                // Fall back to the unlinked path so a missing or unreadable
                // child manifest does not abort the parent signing. Verifiers
                // will report `ingredient.unknownProvenance` for the child.
                tracing::warn!(
                    "Failed to read child manifest for ingredient at {}: {error}",
                    manifest_source.display()
                );
            }
        }
    }

    builder.add_ingredient(build_ingredient(snapshot, index)?);
    Ok(())
}

fn attach_ingredient_with_manifest(
    builder: &mut Builder,
    snapshot: &IngredientSnapshot,
    index: usize,
    manifest_source: &Path,
) -> Result<()> {
    if !is_c2pa_sidecar(manifest_source) {
        match attach_ingredient_from_asset_file(builder, snapshot, index, manifest_source) {
            Ok(()) => return Ok(()),
            Err(error) => {
                tracing::debug!(
                    "Could not load ingredient manifest from asset file at {}: {error}",
                    manifest_source.display()
                );
            }
        }
    }

    let format = manifest_stream_format(manifest_source, snapshot.media_type.as_deref())?;

    let title = snapshot
        .title
        .clone()
        .unwrap_or_else(|| "ingredient".to_string());
    let ingredient_format = snapshot
        .media_type
        .clone()
        .unwrap_or_else(|| format.clone());

    // Minimal seed JSON; remaining fields are mutated on the returned
    // ingredient after the SDK populates manifest_data and instance_id from
    // the stream.
    let seed = json!({
        "label": ingredient_label(snapshot, index),
        "title": title,
        "format": ingredient_format,
    });

    let mut file = File::open(manifest_source)?;
    let ingredient = builder.add_ingredient_from_stream(seed.to_string(), &format, &mut file)?;

    apply_ingredient_metadata(ingredient, snapshot);

    Ok(())
}

fn attach_ingredient_from_asset_file(
    builder: &mut Builder,
    snapshot: &IngredientSnapshot,
    index: usize,
    manifest_source: &Path,
) -> Result<()> {
    let mut ingredient = build_ingredient_from_asset_file(snapshot, index, manifest_source)?;

    apply_ingredient_metadata(&mut ingredient, snapshot);

    builder.add_ingredient(ingredient);
    Ok(())
}

fn build_ingredient_from_asset_file(
    snapshot: &IngredientSnapshot,
    index: usize,
    manifest_source: &Path,
) -> Result<Ingredient> {
    #[allow(deprecated)]
    let source_ingredient = Ingredient::from_file(manifest_source)?;
    let mut validation_results = source_ingredient
        .validation_results()
        .map(serde_json::to_value)
        .transpose()?;
    let manifest_data = source_ingredient
        .manifest_data()
        .map(std::borrow::Cow::into_owned)
        .or_else(|| fs::read(media::sidecar_path(manifest_source)).ok())
        .ok_or_else(|| {
            Error::other(format!(
                "asset file has no readable C2PA manifest: {}",
                manifest_source.display()
            ))
        })?;
    if validation_results.is_none() {
        match validation_results_from_manifest_source(
            manifest_source,
            &manifest_data,
            snapshot.media_type.as_deref(),
        ) {
            Ok(results) => validation_results = results,
            Err(error) => {
                tracing::debug!(
                    "Could not validate ingredient manifest source at {}: {error}",
                    manifest_source.display()
                );
            }
        }
    }
    if validation_results.is_none() && snapshot.relationship == IngredientRelationship::InputTo {
        validation_results = Some(json!({}));
    }

    let mut seed = serde_json::to_value(&source_ingredient)?;
    seed["label"] = json!(ingredient_label(snapshot, index));
    if let Some(title) = &snapshot.title {
        seed["title"] = json!(title);
    }
    if let Some(format) = &snapshot.media_type {
        seed["format"] = json!(format);
    }
    if let Some(validation_results) = validation_results {
        seed["validation_results"] = validation_results;
    }

    let mut ingredient = Ingredient::from_json(&seed.to_string())?;
    ingredient.set_manifest_data(manifest_data)?;

    Ok(ingredient)
}

fn validation_results_from_manifest_source(
    manifest_source: &Path,
    manifest_data: &[u8],
    media_type: Option<&str>,
) -> Result<Option<Value>> {
    let media_type = media_type.map_or_else(
        || media::guess_media_type(manifest_source),
        |media_type| Ok(media_type.to_string()),
    )?;
    let mut asset = File::open(manifest_source)?;
    let reader = Reader::from_context(Context::new()).with_manifest_data_and_stream(
        manifest_data,
        &media_type,
        &mut asset,
    )?;

    reader
        .validation_results()
        .map(serde_json::to_value)
        .transpose()
        .map_err(Error::from)
}

/// Format string to pass to the c2pa SDK when reading a child manifest from a
/// stream. `.c2pa` sidecars are loaded as `application/c2pa`; everything else
/// is loaded as the asset's own media type so the SDK can find the embedded
/// JUMBF box.
fn manifest_stream_format(path: &Path, ingredient_media_type: Option<&str>) -> Result<String> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("c2pa"))
    {
        return Ok("application/c2pa".to_string());
    }

    if let Some(media_type) = ingredient_media_type {
        return Ok(media_type.to_string());
    }

    media::guess_media_type(path)
}

fn is_c2pa_sidecar(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("c2pa"))
}

/// Build a c2pa-rs [`Ingredient`] from a Stencila [`IngredientSnapshot`].
///
/// Used as a fallback when no child manifest can be linked. The returned
/// ingredient has no `manifest_data`, so verifiers will surface
/// `ingredient.unknownProvenance` for it.
fn build_ingredient(snapshot: &IngredientSnapshot, index: usize) -> Result<Ingredient> {
    let title = snapshot
        .title
        .clone()
        .unwrap_or_else(|| "ingredient".to_string());
    let format = snapshot
        .media_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let seed = json!({
        "label": ingredient_label(snapshot, index),
        "title": title,
        "format": format,
    });

    let mut ingredient = Ingredient::from_json(&seed.to_string())?;
    apply_ingredient_metadata(&mut ingredient, snapshot);
    Ok(ingredient)
}

fn apply_ingredient_metadata(ingredient: &mut Ingredient, snapshot: &IngredientSnapshot) {
    ingredient.set_relationship(snapshot_relationship(snapshot.relationship));
    if let Some(hash) = &snapshot.content_digest {
        ingredient.set_hash(hash.clone());
    }
    if let Some(uri) = &snapshot.informational_uri {
        ingredient.set_informational_uri(uri.clone());
    }
    if let Some(description) = &snapshot.description {
        ingredient.set_description(description.clone());
    }
    apply_ingredient_thumbnail(ingredient, snapshot.thumbnail.as_ref());
}

fn apply_ingredient_thumbnail(
    ingredient: &mut Ingredient,
    thumbnail: Option<&IngredientThumbnailSnapshot>,
) {
    let Some(thumbnail) = thumbnail else {
        return;
    };

    match read_ingredient_thumbnail(thumbnail) {
        Ok(Some((format, bytes))) => {
            // Best-effort metadata: a thumbnail should not make signing fail.
            let _ = ingredient.set_thumbnail(format, bytes);
        }
        Ok(None) => {}
        Err(error) => {
            tracing::debug!("Could not read Content Credentials ingredient thumbnail: {error}");
        }
    }
}

fn read_ingredient_thumbnail(
    thumbnail: &IngredientThumbnailSnapshot,
) -> Result<Option<(String, Vec<u8>)>> {
    if let Some(bytes) = &thumbnail.bytes {
        let Some(format) = thumbnail.media_type.as_deref().and_then(thumbnail_format) else {
            return Ok(None);
        };

        if bytes.is_empty() || bytes.len() as u64 > MAX_THUMBNAIL_BYTES {
            return Ok(None);
        }

        return Ok(Some((format.to_string(), bytes.clone())));
    }

    let Some(path) = thumbnail.source_path.as_deref() else {
        return Ok(None);
    };

    let media_type = thumbnail
        .media_type
        .clone()
        .map_or_else(|| media::guess_media_type(path), Ok)?;
    let Some(format) = thumbnail_format(&media_type) else {
        return Ok(None);
    };

    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_THUMBNAIL_BYTES {
        return Ok(None);
    }

    Ok(Some((format.to_string(), fs::read(path)?)))
}

fn snapshot_relationship(relationship: IngredientRelationship) -> Relationship {
    match relationship {
        IngredientRelationship::ParentOf => Relationship::ParentOf,
        IngredientRelationship::InputTo => Relationship::InputTo,
        IngredientRelationship::ComponentOf => Relationship::ComponentOf,
    }
}

/// Software agent value used in `c2pa.actions.v2` actions.
///
/// Returned as a `ClaimGeneratorInfo` object (`{name, version}`) rather than a
/// joined string so the agent is machine-parseable and consistent with
/// `claim_generator_info` at the manifest level.
fn software_agent_value(assertion: &ProvenanceAssertion) -> Value {
    json!({
        "name": assertion.producer.name,
        "version": assertion.producer.version,
    })
}

fn action_description(assertion: &ProvenanceAssertion) -> String {
    assertion
        .activities
        .iter()
        .rev()
        .find_map(|activity| activity.name.clone())
        .unwrap_or_else(|| "Create asset".to_string())
}

fn action_timestamp(assertion: &ProvenanceAssertion) -> Option<String> {
    assertion.activities.iter().rev().find_map(|activity| {
        activity
            .ended_at
            .clone()
            .or_else(|| activity.started_at.clone())
    })
}

fn action_parameters(
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
    include_parent_ingredients: bool,
) -> Value {
    let mut parameters = json!({
        "org.stencila.assetType": assertion.asset.asset_type,
        "org.stencila.mediaType": assertion.asset.media_type,
    });

    if let Some(role) = &assertion.asset.role {
        parameters["org.stencila.assetRole"] = json!(role);
    }
    if let Some(codec) = &assertion.producer.codec {
        parameters["org.stencila.codec"] = json!(codec);
    }
    if let Some(renderer) = &assertion.producer.renderer {
        parameters["org.stencila.renderer"] = json!(renderer);
    }

    let source_ingredient_ids: Vec<String> = ingredients
        .iter()
        .enumerate()
        .filter(|(_, ingredient)| {
            ingredient.relationship == IngredientRelationship::InputTo
                || (include_parent_ingredients
                    && ingredient.relationship == IngredientRelationship::ParentOf)
        })
        .map(|(index, ingredient)| ingredient_label(ingredient, index))
        .collect();

    if !source_ingredient_ids.is_empty() {
        parameters["ingredientIds"] = json!(source_ingredient_ids);
    }

    parameters
}

fn placed_actions(
    assertion: &ProvenanceAssertion,
    ingredients: &[IngredientSnapshot],
) -> Vec<Value> {
    ingredients
        .iter()
        .enumerate()
        .filter(|(_, ingredient)| ingredient.relationship == IngredientRelationship::ComponentOf)
        .map(|(index, ingredient)| {
            let mut action = json!({
                "action": "c2pa.placed",
                "description": ingredient
                    .title
                    .as_deref()
                    .map_or_else(|| "Place component".to_string(), |title| format!("Place {title}")),
                "softwareAgent": software_agent_value(assertion),
                "parameters": {
                    "ingredientIds": [ingredient_label(ingredient, index)]
                }
            });

            if let Some(when) = action_timestamp(assertion) {
                action["when"] = json!(when);
            }

            action
        })
        .collect()
}

fn ingredient_label(snapshot: &IngredientSnapshot, index: usize) -> String {
    snapshot
        .label
        .as_deref()
        .filter(|label| !label.trim().is_empty())
        .map_or_else(
            || format!("stencila-ingredient-{index}"),
            ToString::to_string,
        )
}

fn producer_software_agent(assertion: &ProvenanceAssertion) -> String {
    format!("{} {}", assertion.producer.name, assertion.producer.version)
}

fn standard_asset_type(assertion: &ProvenanceAssertion) -> String {
    let stencila_type = assertion
        .asset
        .role
        .as_deref()
        .unwrap_or(&assertion.asset.asset_type)
        .trim()
        .replace(|ch: char| !ch.is_ascii_alphanumeric(), "-")
        .trim_matches('-')
        .to_ascii_lowercase();

    if stencila_type.is_empty() {
        "org.stencila.asset.asset".to_string()
    } else {
        format!("org.stencila.asset.{stencila_type}")
    }
}
