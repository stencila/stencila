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
use stencila_schema::{Node, NodeId};

use crate::{
    CredentialProducer, CredentialProfile, CredentialSignerConfig, Error, IngredientRelationship,
    IngredientSnapshot, ManifestKind, Result, SignAssetRequest, SignedAsset, SourceRangeSnapshot,
    media,
};

use self::{
    ingredients::{
        add_source_and_executed_ingredients, source_ingredient_manifest, source_ingredient_snapshot,
    },
    snapshot::{ExportSnapshotOptions, build_export_snapshot},
};

mod environment;
mod execution;
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
    role: Option<String>,
    title: Option<String>,
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
        info,
    } = request;

    let signer_config = CredentialSignerConfig::resolve(None, None)?;
    let producer = CredentialProducer::new(signer_config);
    let profile_label = credential_profile.label();

    if !output_path.is_file() {
        return Err(Error::InputNotFound(output_path.to_path_buf()));
    }

    let source_ingredient = source_ingredient_snapshot(source_path);

    let mut side_targets = Vec::new();
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    seen.insert(output_path.to_path_buf());
    for asset in &info.assets {
        if !seen.insert(asset.path.clone()) {
            continue;
        }
        side_targets.push(SideAssetTarget {
            path: asset.path.clone(),
            originating_id: asset.node_id.clone(),
            role: asset.role.clone(),
            title: asset.title.clone(),
        });
    }

    let source_manifest = if source_ingredient.is_some() {
        match source_ingredient_manifest(
            &producer,
            source_path,
            source_ingredient.as_ref(),
            credential_profile,
        )
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
    } else {
        None
    };
    let source_manifest_path = source_manifest
        .as_ref()
        .map(|manifest| manifest.asset_path.as_path());

    let mut new_sidecars: Vec<EncodedAsset> = Vec::new();
    let mut component_ingredients: Vec<IngredientSnapshot> = Vec::new();

    for (component_index, target) in side_targets.into_iter().enumerate() {
        let SideAssetTarget {
            path: asset_path,
            originating_id,
            role: asset_role,
            title: asset_title,
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
                codec_name: Some(codec_name),
                profile: credential_profile,
            },
        );
        let _temporary_ingredient_manifests = add_source_and_executed_ingredients(
            &producer,
            &mut provenance,
            source_ingredient.clone(),
            source_path,
            source_manifest_path,
            credential_profile,
        )
        .await?;

        let signed = producer
            .sign_exported_asset(SignAssetRequest {
                input_path: asset_path.clone(),
                media_type: media_type.clone(),
                title: asset_title.clone(),
                credential_profile,
                provenance: Some(provenance),
                ..Default::default()
            })
            .await?;

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

        push_sidecar_asset_once(
            signed.sidecar_path.as_deref(),
            &info.assets,
            &mut new_sidecars,
        );

        component_ingredients.push(IngredientSnapshot {
            label: Some(format!("component-{component_index}")),
            title: asset_title.or_else(|| {
                asset_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(ToString::to_string)
            }),
            media_type,
            content_digest: Some(signed.signed_asset_digest),
            relationship: IngredientRelationship::ComponentOf,
            manifest_source: Some(manifest_source),
            ..Default::default()
        });
    }

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
                codec_name: Some(codec_name),
                profile: credential_profile,
            },
        );
        let _temporary_ingredient_manifests = add_source_and_executed_ingredients(
            &producer,
            &mut provenance,
            source_ingredient,
            source_path,
            source_manifest_path,
            credential_profile,
        )
        .await?;
        provenance.ingredients.extend(component_ingredients);

        let signed = producer
            .sign_exported_asset(SignAssetRequest {
                input_path: output_path.to_path_buf(),
                media_type,
                credential_profile,
                provenance: Some(provenance),
                ..Default::default()
            })
            .await?;

        let primary_asset = EncodedAsset {
            path: output_path.to_path_buf(),
            node_id: node.node_id().map(|id| id.uid_str().to_string()),
            node_type: Some(node.node_type().to_string()),
            role: Some("document".to_string()),
            title: None,
            signed: true,
            manifest_kind: Some(manifest_kind_label(signed.manifest_kind).to_string()),
            manifest_id: signed.manifest_id.clone(),
            sidecar_path: signed.sidecar_path.clone(),
            credential_profile: Some(signed.credential_profile.label().to_string()),
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

fn manifest_kind_label(kind: ManifestKind) -> &'static str {
    match kind {
        ManifestKind::Embedded => "embedded",
        ManifestKind::Sidecar => "sidecar",
    }
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
