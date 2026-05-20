//! Sign Stencila codec exports with Content Credentials.
//!
//! This module is the public surface for export-time signing. It coordinates
//! side-asset signing, primary asset signing, and provenance chaining while the
//! private child modules project Stencila nodes, source facts, execution facts,
//! and ingredients into snapshot values.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use stencila_codec_info::{EncodeInfo, EncodedAsset};
use stencila_node_media::{extract_media_with_paths, reference_media_with_paths};
use stencila_schema::{Node, NodeId, NodeType, Visitor, WalkControl};
use tempfile::{TempDir, tempdir};

use crate::{
    CredentialCloudSigningConfig, CredentialProducer, CredentialProfile, CredentialSigningConfig,
    CredentialSigningMode, Error, IngredientSnapshot, ManifestKind, ProvenanceSnapshot, Result,
    SignAssetRequest, SignedAsset, SourceRangeSnapshot, media, producer,
};

use self::{
    components::{ComponentIngredient, signed_component_ingredient},
    figures::group_figure_component_ingredients,
    ingredients::{
        add_environment_ingredient, add_source_and_executed_ingredients,
        source_ingredient_manifest, source_ingredient_snapshot,
    },
    snapshot::{ExportSnapshotOptions, build_export_snapshot},
};

mod components;
mod environment;
mod execution;
mod figures;
mod ingredients;
mod snapshot;
mod source;

/// Inputs for signing every filesystem path emitted by a codec export.
///
/// The request keeps the signing layer independent of codec dispatch while still
/// carrying the document tree and encode metadata needed to attach per-node
/// provenance to side assets.
pub struct ExportSigningRequest<'a> {
    /// Stabilized and stripped document node used for the export.
    pub node: &'a Node,

    /// Name of the codec that produced the export.
    pub codec_name: &'a str,

    /// Primary exported document path.
    pub output_path: &'a Path,

    /// Original source document path, when the export came from a path decode.
    pub source_path: Option<&'a Path>,

    /// Source ranges keyed by full node id.
    pub source_ranges: Option<&'a BTreeMap<String, SourceRangeSnapshot>>,

    /// Media type to use for the primary output when it cannot be inferred.
    pub media_type_hint: Option<String>,

    /// Privacy projection profile used by the signer.
    pub credential_profile: CredentialProfile,

    /// Signing backend configuration.
    ///
    /// When absent, the automatic signing backend is used. CLI and Cloud
    /// callers can pass an explicit backend to avoid hidden fallback behavior.
    pub signing_config: Option<CredentialSigningConfig>,

    /// Whether Cloud soft binding registration was requested.
    pub soft_binding: bool,

    /// Encoding metadata to update with signing results.
    pub info: &'a mut EncodeInfo,
}

/// Inputs for signing filesystem assets emitted by a codec export without
/// signing the primary output document.
///
/// Site rendering uses this to sign media before DOM encoding, so the final
/// HTML can carry advisory Content Credentials attributes for those media
/// objects without requiring a page-level manifest.
pub struct AssetSigningRequest<'a> {
    /// Stabilized document node used for the export.
    pub node: &'a Node,

    /// Name of the codec that will produce the export.
    pub codec_name: &'a str,

    /// Original source document path, when the export came from a path decode.
    pub source_path: Option<&'a Path>,

    /// Source ranges keyed by full node id.
    pub source_ranges: Option<&'a BTreeMap<String, SourceRangeSnapshot>>,

    /// Privacy projection profile used by the signer.
    pub credential_profile: CredentialProfile,

    /// Signing backend configuration.
    ///
    /// When absent, the automatic signing backend is used.
    pub signing_config: Option<CredentialSigningConfig>,

    /// Whether Cloud soft binding registration was requested.
    pub soft_binding: bool,

    /// Encoding metadata to update with signing results.
    pub info: &'a mut EncodeInfo,
}

/// Side asset selected from codec encode metadata for signing.
///
/// The signing loop snapshots these fields before mutating `EncodeInfo` so that
/// adding sidecar records or signed metadata does not change which assets are
/// considered part of the original export.
struct SideAssetTarget {
    path: PathBuf,
    originating_id: Option<String>,
    node_type: Option<String>,
    role: Option<String>,
    title: Option<String>,
    description: Option<String>,
    emitted: bool,
}

struct SignedSideAssets {
    component_ingredients: Vec<ComponentIngredient>,
    new_sidecars: Vec<EncodedAsset>,
    temporary_static_component_dirs: Vec<TempDir>,
    component_index: usize,
}

/// Sign only the filesystem assets represented in [`EncodeInfo`].
///
/// This mutates matching asset rows with manifest metadata and appends generated
/// `.c2pa` sidecar rows. It intentionally does not sign the primary output.
///
/// # Errors
///
/// Returns an error if signing credentials cannot be resolved or an emitted
/// asset cannot be signed.
#[allow(clippy::too_many_lines)]
pub async fn sign_encoded_assets(request: AssetSigningRequest<'_>) -> Result<()> {
    let AssetSigningRequest {
        node,
        codec_name,
        source_path,
        source_ranges,
        credential_profile,
        signing_config,
        soft_binding,
        info,
    } = request;

    let producer = producer_for_config(signing_config, soft_binding)?;
    let source_ingredient = source_ingredient_snapshot(source_path);
    let source_manifest = source_manifest(
        &producer,
        source_path,
        source_ingredient.as_ref(),
        credential_profile,
    )
    .await;
    let source_manifest_path = source_manifest
        .as_ref()
        .map(|manifest| manifest.asset_path.as_path());

    let signed = Box::pin(sign_side_assets(
        &producer,
        node,
        codec_name,
        source_path,
        source_ranges,
        source_ingredient,
        source_manifest_path,
        credential_profile,
        info,
        false,
        soft_binding,
        BTreeSet::new(),
        0,
    ))
    .await?;

    info.assets.extend(signed.new_sidecars);

    tracing::debug!(
        profile = credential_profile.label(),
        "Signed codec side assets with Content Credentials"
    );

    Ok(())
}

/// Sign every path emitted by a Stencila codec export.
///
/// Side assets are signed before the primary output so the primary manifest can
/// declare them as `componentOf` ingredients with hashes over the signed bytes.
/// Signing mutates side assets in place, inserts a signed primary document record
/// at the front of [`EncodeInfo::assets`], and appends any generated `.c2pa`
/// sidecar records.
///
/// # Errors
///
/// Returns an error if signing credentials cannot be resolved, an asset cannot
/// be signed, the primary output is not a file, or temporary ingredient files
/// cannot be written.
#[allow(clippy::too_many_lines)]
pub async fn sign_encoded_export(request: ExportSigningRequest<'_>) -> Result<()> {
    let ExportSigningRequest {
        node,
        codec_name,
        output_path,
        source_path,
        source_ranges,
        media_type_hint,
        credential_profile,
        signing_config,
        soft_binding,
        info,
    } = request;

    let producer = producer_for_config(signing_config, soft_binding)?;
    let profile_label = credential_profile.label();

    if !output_path.is_file() {
        return Err(Error::InputNotFound(output_path.to_path_buf()));
    }

    let source_ingredient = source_ingredient_snapshot(source_path);
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    seen.insert(output_path.to_path_buf());

    let source_manifest = source_manifest(
        &producer,
        source_path,
        source_ingredient.as_ref(),
        credential_profile,
    )
    .await;
    let source_manifest_path = source_manifest
        .as_ref()
        .map(|manifest| manifest.asset_path.as_path());

    let signed = Box::pin(sign_side_assets(
        &producer,
        node,
        codec_name,
        source_path,
        source_ranges,
        source_ingredient.clone(),
        source_manifest_path,
        credential_profile,
        info,
        true,
        soft_binding,
        seen,
        0,
    ))
    .await?;
    let mut new_sidecars = signed.new_sidecars;
    let mut component_ingredients = signed.component_ingredients;
    let mut component_index = signed.component_index;

    // Keep temp dirs alive until the primary manifest has linked the temporary
    // child manifests referenced by `component_ingredients`.
    let _temporary_static_component_dirs = signed.temporary_static_component_dirs;
    let _temporary_component_dir = if supports_embedded_component_extraction(codec_name) {
        let (embedded_components, temporary_component_dir) =
            Box::pin(embedded_component_ingredients(
                &producer,
                node,
                output_path,
                source_ranges,
                source_path,
                source_ingredient.clone(),
                source_manifest_path,
                codec_name,
                credential_profile,
                component_index,
            ))
            .await?;
        component_ingredients.extend(embedded_components);
        component_index = component_ingredients.len();
        temporary_component_dir
    } else {
        None
    };

    {
        let media_type = match media::guess_media_type(output_path) {
            Ok(media_type) => Some(media_type),
            Err(error) => {
                tracing::debug!("{error}");
                media_type_hint
            }
        };

        let mut provenance = build_export_snapshot(
            node,
            node,
            output_path,
            ExportSnapshotOptions {
                source_ranges,
                source_path,
                primary: true,
                asset_role: None,
                asset_title: None,
                asset_description: None,
                codec_name: Some(codec_name),
                profile: credential_profile,
            },
        );
        let _temporary_ingredient_manifests = Box::pin(add_source_and_executed_ingredients(
            &producer,
            &mut provenance,
            source_ingredient.clone(),
            source_path,
            source_manifest_path,
            credential_profile,
        ))
        .await?;
        let _temporary_environment_manifest = if root_depends_on_execution_environment(node) {
            Box::pin(add_environment_ingredient(
                &producer,
                &mut provenance,
                credential_profile,
            ))
            .await?
        } else {
            None
        };
        let (component_ingredients, temporary_parent_dirs) =
            Box::pin(group_figure_component_ingredients(
                &producer,
                node,
                source_ranges,
                source_path,
                codec_name,
                credential_profile,
                component_ingredients,
                component_index,
            ))
            .await?;
        let _temporary_parent_dirs = temporary_parent_dirs;
        provenance.ingredients.extend(
            component_ingredients
                .into_iter()
                .map(|component| component.ingredient),
        );

        let signed = add_soft_binding_warning(
            producer
                .sign_exported_asset(SignAssetRequest {
                    input_path: output_path.to_path_buf(),
                    media_type,
                    credential_profile,
                    provenance: Some(provenance),
                    ..Default::default()
                })
                .await?,
            soft_binding,
        );

        let primary_asset = EncodedAsset {
            path: output_path.to_path_buf(),
            node_id: node.node_id().map(|id| id.uid_str().to_string()),
            node_type: Some(node.node_type().to_string()),
            role: Some("document".to_string()),
            title: None,
            description: None,
            signed: true,
            manifest_kind: Some(manifest_kind_label(signed.manifest_kind).to_string()),
            manifest_id: signed.manifest_id.clone(),
            sidecar_path: signed.sidecar_path.clone(),
            credential_profile: Some(signed.credential_profile.label().to_string()),
            c2pa: signed.c2pa.clone(),
            signing_warnings: signed.warnings.clone(),
        };
        push_sidecar_asset_once(
            signed.sidecar_path.as_deref(),
            &info.assets,
            &mut new_sidecars,
        );
        info.assets.insert(0, primary_asset);
    }

    info.assets.extend(new_sidecars);

    tracing::debug!(
        profile = profile_label,
        "Signed codec export with Content Credentials"
    );

    Ok(())
}

fn producer_for_config(
    signing_config: Option<CredentialSigningConfig>,
    soft_binding: bool,
) -> Result<CredentialProducer> {
    let signing_config = match signing_config {
        Some(config) => config,
        None => CredentialSigningConfig::resolve_auto_with_cloud_config(
            CredentialCloudSigningConfig::resolve(),
        )?,
    };

    let signing_config = with_soft_binding(signing_config, soft_binding);

    Ok(CredentialProducer::new(signing_config))
}

fn with_soft_binding(
    signing_config: CredentialSigningConfig,
    soft_binding: bool,
) -> CredentialSigningConfig {
    match signing_config {
        CredentialSigningConfig::Local(local) => CredentialSigningConfig::Local(local),
        CredentialSigningConfig::Cloud(cloud) => {
            CredentialSigningConfig::Cloud(cloud.with_register_soft_binding(soft_binding))
        }
        CredentialSigningConfig::Auto { cloud, local } => CredentialSigningConfig::Auto {
            cloud: cloud.with_register_soft_binding(soft_binding),
            local,
        },
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
async fn sign_side_assets(
    producer: &CredentialProducer,
    node: &Node,
    codec_name: &str,
    source_path: Option<&Path>,
    source_ranges: Option<&BTreeMap<String, SourceRangeSnapshot>>,
    source_ingredient: Option<IngredientSnapshot>,
    source_manifest_path: Option<&Path>,
    credential_profile: CredentialProfile,
    info: &mut EncodeInfo,
    include_referenced_media: bool,
    soft_binding: bool,
    mut seen: BTreeSet<PathBuf>,
    mut component_index: usize,
) -> Result<SignedSideAssets> {
    let mut side_targets = Vec::new();
    for asset in &info.assets {
        if !seen.insert(asset.path.clone()) {
            continue;
        }
        side_targets.push(SideAssetTarget {
            path: asset.path.clone(),
            originating_id: asset.node_id.clone(),
            node_type: asset.node_type.clone(),
            role: asset.role.clone(),
            title: asset.title.clone(),
            description: asset.description.clone(),
            emitted: true,
        });
    }

    if include_referenced_media {
        match reference_media_with_paths(node, source_path) {
            Ok(assets) => {
                for asset in assets {
                    if !seen.insert(asset.path.clone()) {
                        continue;
                    }
                    side_targets.push(SideAssetTarget {
                        path: asset.path,
                        originating_id: asset.node_id,
                        node_type: asset.node_type,
                        role: asset.role,
                        title: asset.title,
                        description: asset.description,
                        emitted: false,
                    });
                }
            }
            Err(error) => {
                tracing::warn!(
                    "Could not discover referenced media for Content Credentials: {error}"
                );
            }
        }
    }

    let mut new_sidecars: Vec<EncodedAsset> = Vec::new();
    let mut component_ingredients: Vec<ComponentIngredient> = Vec::new();
    let mut temporary_static_component_dirs: Vec<TempDir> = Vec::new();

    for target in side_targets {
        let SideAssetTarget {
            path: asset_path,
            originating_id,
            node_type: target_node_type,
            role: asset_role,
            title: asset_title,
            description: asset_description,
            emitted,
        } = target;

        if !asset_path.is_file() {
            tracing::warn!(
                asset = %asset_path.display(),
                "Skipping Content Credentials for emitted asset because it is not a file"
            );
            continue;
        }

        let media_type = match media::guess_media_type(&asset_path) {
            Ok(media_type) => Some(media_type),
            Err(error) => {
                tracing::warn!(
                    "Skipping content credentials for asset with unknown media type: {}",
                    asset_path.display()
                );
                tracing::debug!("{error}");
                continue;
            }
        };

        if media::has_c2pa_manifest(&asset_path, media_type.as_deref()) {
            let media_type_value = media_type.as_deref().unwrap_or("application/octet-stream");
            let sidecar = media::sidecar_path(&asset_path);
            let (manifest_id, c2pa, signing_warnings, sidecar_path) =
                if media::could_have_embedded(media_type_value) {
                    let (manifest_id, c2pa, warnings) =
                        producer::read_signed_manifest_info(&asset_path, None, media_type_value);
                    if manifest_id.is_some() || !sidecar.exists() {
                        (manifest_id, c2pa, warnings, None)
                    } else {
                        let (manifest_id, c2pa, warnings) = producer::read_signed_manifest_info(
                            &asset_path,
                            Some(&sidecar),
                            media_type_value,
                        );
                        (manifest_id, c2pa, warnings, Some(sidecar.as_path()))
                    }
                } else {
                    let sidecar_path = sidecar.exists().then_some(sidecar.as_path());
                    let (manifest_id, c2pa, warnings) = producer::read_signed_manifest_info(
                        &asset_path,
                        sidecar_path,
                        media_type_value,
                    );
                    (manifest_id, c2pa, warnings, sidecar_path)
                };

            if let Some(asset) = info
                .assets
                .iter_mut()
                .find(|asset| asset.path == asset_path)
            {
                asset.signed = true;
                asset.manifest_kind = Some(
                    if sidecar_path.is_some() {
                        "sidecar"
                    } else {
                        "embedded"
                    }
                    .to_string(),
                );
                asset.manifest_id.clone_from(&manifest_id);
                asset.sidecar_path = sidecar_path.map(Path::to_path_buf);
                asset.credential_profile = Some(credential_profile.label().to_string());
                asset.c2pa.clone_from(&c2pa);
                asset.signing_warnings.clone_from(&signing_warnings);
            }
            component_ingredients.push(signed_component_ingredient(
                component_index,
                originating_id,
                asset_title,
                asset_description,
                &asset_path,
                media_type,
                media::sha256_file(&asset_path)?,
                asset_path.clone(),
                target_node_type.as_deref(),
            ));
            component_index += 1;
            continue;
        }

        let owned_subject = originating_id
            .as_deref()
            .and_then(|id| id.parse::<NodeId>().ok())
            .and_then(|id| stencila_node_find::find(node, id));
        let subject = owned_subject.as_ref().unwrap_or(node);

        let mut provenance = build_export_snapshot(
            node,
            subject,
            &asset_path,
            ExportSnapshotOptions {
                source_ranges,
                source_path,
                primary: false,
                asset_role: asset_role.as_deref(),
                asset_title: asset_title.as_deref(),
                asset_description: asset_description.as_deref(),
                codec_name: Some(codec_name),
                profile: credential_profile,
            },
        );
        let _temporary_ingredient_manifests = Box::pin(add_source_and_executed_ingredients(
            producer,
            &mut provenance,
            source_ingredient.clone(),
            source_path,
            source_manifest_path,
            credential_profile,
        ))
        .await?;

        let static_component_dir = if emitted { None } else { Some(tempdir()?) };
        let output_path = static_component_dir.as_ref().map(|dir| {
            dir.path().join(
                asset_path
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("component")),
            )
        });

        let signed = add_soft_binding_warning(
            producer
                .sign_exported_asset(SignAssetRequest {
                    input_path: asset_path.clone(),
                    output_path: output_path.clone(),
                    media_type: media_type.clone(),
                    title: asset_title.clone(),
                    credential_profile,
                    provenance: Some(provenance),
                })
                .await?,
            soft_binding,
        );

        let manifest_source = signed
            .sidecar_path
            .clone()
            .unwrap_or_else(|| signed.asset_path.clone());

        if let Some(asset) = info
            .assets
            .iter_mut()
            .find(|asset| asset.path == asset_path)
        {
            apply_signed_asset_metadata(asset, &signed);
        }

        if emitted {
            push_sidecar_asset_once(
                signed.sidecar_path.as_deref(),
                &info.assets,
                &mut new_sidecars,
            );
        }

        component_ingredients.push(signed_component_ingredient(
            component_index,
            originating_id,
            asset_title,
            asset_description,
            &asset_path,
            media_type,
            signed.signed_asset_digest,
            manifest_source,
            target_node_type.as_deref(),
        ));
        component_index += 1;

        if let Some(dir) = static_component_dir {
            temporary_static_component_dirs.push(dir);
        }
    }

    Ok(SignedSideAssets {
        component_ingredients,
        new_sidecars,
        temporary_static_component_dirs,
        component_index,
    })
}

async fn source_manifest(
    producer: &CredentialProducer,
    source_path: Option<&Path>,
    source_ingredient: Option<&IngredientSnapshot>,
    credential_profile: CredentialProfile,
) -> Option<ingredients::TemporaryIngredientManifest> {
    source_ingredient?;

    match source_ingredient_manifest(producer, source_path, source_ingredient, credential_profile)
        .await
    {
        Ok(manifest) => manifest,
        Err(error) => {
            tracing::warn!(
                "Could not create a source manifest for Content Credentials ingredient: {error}"
            );
            None
        }
    }
}

/// Sign embedded data-URI media as temporary component ingredients.
///
/// Some exports, notably PDF, embed rendered media directly into the primary
/// asset and therefore do not report side assets in [`EncodeInfo`]. Extracting
/// those data URIs into a temporary directory lets the primary manifest still
/// describe generated figures as `componentOf` ingredients with their own C2PA
/// manifests, without leaving extra files next to the rendered document.
#[allow(clippy::too_many_arguments)]
async fn embedded_component_ingredients(
    producer: &CredentialProducer,
    node: &Node,
    output_path: &Path,
    source_ranges: Option<&BTreeMap<String, SourceRangeSnapshot>>,
    source_path: Option<&Path>,
    source_ingredient: Option<IngredientSnapshot>,
    source_manifest_path: Option<&Path>,
    codec_name: &str,
    credential_profile: CredentialProfile,
    mut component_index: usize,
) -> Result<(Vec<ComponentIngredient>, Option<TempDir>)> {
    let temp_dir = tempdir()?;
    let mut node = node.clone();
    let assets = extract_media_with_paths(&mut node, Some(output_path), temp_dir.path()).map_err(
        |error| {
            Error::other(format!(
                "could not extract embedded component media: {error}"
            ))
        },
    )?;
    if assets.is_empty() {
        return Ok((Vec::new(), None));
    }

    let mut component_ingredients = Vec::new();

    for asset in assets {
        if !asset.path.is_file() {
            tracing::warn!(
                asset = %asset.path.display(),
                "Skipping Content Credentials component ingredient because extracted media is not a file"
            );
            continue;
        }

        let media_type = match media::guess_media_type(&asset.path) {
            Ok(media_type) => Some(media_type),
            Err(error) => {
                tracing::warn!(
                    "Skipping Content Credentials component ingredient with unknown media type: {}",
                    asset.path.display()
                );
                tracing::debug!("{error}");
                continue;
            }
        };

        let owned_subject = asset
            .node_id
            .as_deref()
            .and_then(|id| id.parse::<NodeId>().ok())
            .and_then(|id| stencila_node_find::find(&node, id));
        let subject = owned_subject.as_ref().unwrap_or(&node);

        let mut provenance = build_export_snapshot(
            &node,
            subject,
            &asset.path,
            ExportSnapshotOptions {
                source_ranges,
                source_path,
                primary: false,
                asset_role: asset.role.as_deref(),
                asset_title: asset.title.as_deref(),
                asset_description: asset.description.as_deref(),
                codec_name: Some(codec_name),
                profile: credential_profile,
            },
        );
        scrub_embedded_component_content_urls(&mut provenance);
        let _temporary_ingredient_manifests = Box::pin(add_source_and_executed_ingredients(
            producer,
            &mut provenance,
            source_ingredient.clone(),
            source_path,
            source_manifest_path,
            credential_profile,
        ))
        .await?;

        let signed = producer
            .sign_exported_asset(SignAssetRequest {
                input_path: asset.path.clone(),
                media_type: media_type.clone(),
                title: asset.title.clone(),
                credential_profile,
                provenance: Some(provenance),
                ..Default::default()
            })
            .await?;

        let manifest_source = signed
            .sidecar_path
            .clone()
            .unwrap_or_else(|| signed.asset_path.clone());

        component_ingredients.push(signed_component_ingredient(
            component_index,
            asset.node_id,
            asset.title,
            asset.description,
            &asset.path,
            media_type,
            signed.signed_asset_digest,
            manifest_source,
            asset.node_type.as_deref(),
        ));
        component_index += 1;
    }

    if component_ingredients.is_empty() {
        Ok((component_ingredients, None))
    } else {
        Ok((component_ingredients, Some(temp_dir)))
    }
}

#[must_use]
fn supports_embedded_component_extraction(codec_name: &str) -> bool {
    codec_name.eq_ignore_ascii_case("pdf")
}

fn scrub_embedded_component_content_urls(provenance: &mut ProvenanceSnapshot) {
    // Extracted media files are temporary implementation details used only so
    // the parent manifest can link to a concrete signed child manifest.
    if let Some(output_node) = provenance.output_node.as_mut() {
        output_node.content_url = None;
    }
}

fn manifest_kind_label(kind: ManifestKind) -> &'static str {
    kind.label()
}

fn add_soft_binding_warning(mut signed: SignedAsset, soft_binding: bool) -> SignedAsset {
    if soft_binding && signed.signing_mode == CredentialSigningMode::Local {
        let warning = "Soft binding registration requires Stencila Cloud signing; skipped because the asset was signed locally.";
        tracing::warn!("{warning}");
        signed.warnings.push(warning.to_string());
    }

    signed
}

/// Return whether the root document contains executable code.
///
/// Environment ingredients describe runtime context for executable code. Static
/// document exports can still have source and component ingredients, but adding
/// the renderer environment as an input would overstate what the document bytes
/// depend on.
fn root_depends_on_execution_environment(root: &Node) -> bool {
    struct Finder {
        found: bool,
    }

    impl Visitor for Finder {
        fn enter_struct(&mut self, node_type: NodeType, _node_id: NodeId) -> WalkControl {
            if self.found {
                return WalkControl::Break;
            }

            if matches!(node_type, NodeType::CodeChunk | NodeType::CodeExpression) {
                self.found = true;
                WalkControl::Break
            } else {
                WalkControl::Continue
            }
        }
    }

    let mut finder = Finder { found: false };
    finder.walk(root);
    finder.found
}

/// Copy signing results back onto the original encoded asset record.
///
/// Codecs already use `EncodedAsset` to expose side-asset metadata. Updating the
/// existing row preserves codec-provided attribution while adding the manifest
/// details downstream tooling needs for inspection or publication.
fn apply_signed_asset_metadata(asset: &mut EncodedAsset, signed: &SignedAsset) {
    asset.signed = true;
    asset.manifest_kind = Some(manifest_kind_label(signed.manifest_kind).to_string());
    asset.manifest_id.clone_from(&signed.manifest_id);
    asset.sidecar_path.clone_from(&signed.sidecar_path);
    asset.credential_profile = Some(signed.credential_profile.label().to_string());
    asset.c2pa.clone_from(&signed.c2pa);
    asset.signing_warnings.clone_from(&signed.warnings);
}

/// Append a sidecar asset row unless the path is already represented.
///
/// Sidecars are produced while signing both side assets and the primary asset.
/// Recording each sidecar exactly once keeps encode metadata useful for callers
/// without making them deduplicate manifest files themselves.
fn push_sidecar_asset_once(
    sidecar_path: Option<&Path>,
    existing_assets: &[EncodedAsset],
    new_sidecars: &mut Vec<EncodedAsset>,
) {
    let Some(sidecar_path) = sidecar_path else {
        return;
    };

    if existing_assets
        .iter()
        .any(|asset| asset.path == sidecar_path)
        || new_sidecars.iter().any(|asset| asset.path == sidecar_path)
    {
        return;
    }

    new_sidecars.push(EncodedAsset::sidecar(sidecar_path.to_path_buf()));
}

#[cfg(test)]
mod tests {
    use crate::{
        CredentialCloudSigningConfig, CredentialSigningConfig, DocumentSnapshot, Error,
        ProvenanceSnapshot, Result,
    };

    use super::{
        scrub_embedded_component_content_urls, supports_embedded_component_extraction,
        with_soft_binding,
    };

    #[test]
    fn embedded_component_extraction_is_pdf_only() {
        assert!(supports_embedded_component_extraction("pdf"));
        assert!(supports_embedded_component_extraction("PDF"));
        assert!(!supports_embedded_component_extraction("html"));
        assert!(!supports_embedded_component_extraction("markdown"));
    }

    #[test]
    fn embedded_component_snapshots_do_not_publish_temporary_content_urls() {
        let mut provenance = ProvenanceSnapshot {
            output_node: Some(DocumentSnapshot {
                node_type: "ImageObject".to_string(),
                content_url: Some("../../tmp/stencila-component/image.png".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        scrub_embedded_component_content_urls(&mut provenance);

        assert!(
            provenance
                .output_node
                .as_ref()
                .is_some_and(|node| node.content_url.is_none()),
            "temporary output contentUrl should be removed"
        );
    }

    #[test]
    fn request_soft_binding_updates_cloud_signing_config() -> Result<()> {
        let config = CredentialSigningConfig::Cloud(
            CredentialCloudSigningConfig::resolve().with_register_soft_binding(false),
        );

        let CredentialSigningConfig::Cloud(cloud) = with_soft_binding(config, true) else {
            return Err(Error::other("Cloud signing config should remain Cloud"));
        };

        if !cloud.register_soft_binding {
            return Err(Error::other(
                "soft binding request should enable Cloud registration",
            ));
        }

        Ok(())
    }

    #[test]
    fn request_soft_binding_updates_auto_cloud_signing_config() -> Result<()> {
        let config = CredentialSigningConfig::Auto {
            cloud: CredentialCloudSigningConfig::resolve().with_register_soft_binding(false),
            local: None,
        };

        let CredentialSigningConfig::Auto { cloud, .. } = with_soft_binding(config, true) else {
            return Err(Error::other("Auto signing config should remain Auto"));
        };

        if !cloud.register_soft_binding {
            return Err(Error::other(
                "soft binding request should enable Auto Cloud registration",
            ));
        }

        Ok(())
    }

    #[test]
    fn disabling_soft_binding_updates_cloud_signing_config() -> Result<()> {
        let config = CredentialSigningConfig::Cloud(
            CredentialCloudSigningConfig::resolve().with_register_soft_binding(true),
        );

        let CredentialSigningConfig::Cloud(cloud) = with_soft_binding(config, false) else {
            return Err(Error::other("Cloud signing config should remain Cloud"));
        };

        if cloud.register_soft_binding {
            return Err(Error::other(
                "soft binding request should disable Cloud registration",
            ));
        }

        Ok(())
    }
}
