//! Thin adapter between codec dispatch and Content Credentials export signing.
//!
//! The signing and provenance projection logic lives in
//! `stencila-content-credentials`. This module keeps the codec-specific source
//! range mapping because it needs to re-encode with the selected codec
//! dispatcher.

use std::{collections::BTreeMap, path::Path};

use tokio::fs::read_to_string;

use stencila_codec::{
    CredentialProfile as CodecCredentialProfile, EncodeInfo, EncodeOptions, PoshMap, eyre::Result,
    stencila_format::Format, stencila_schema::Node,
};
use stencila_content_credentials::{
    CredentialProfile, SourceRangeSnapshot,
    export::{
        AssetSigningRequest, ExportSigningRequest, sign_encoded_assets as sign_assets,
        sign_encoded_export as sign_export,
    },
};

/// Sign paths emitted by a codec export, when credentials were requested.
pub(crate) async fn sign_encoded_export(
    node: &Node,
    codec_name: &str,
    path: &Path,
    options: Option<&EncodeOptions>,
    info: &mut EncodeInfo,
) -> Result<()> {
    let Some(credentials) = options.and_then(|options| options.credentials.as_ref()) else {
        return Ok(());
    };

    let source_path = options.and_then(|options| options.from_path.as_deref());
    let source_ranges = source_range_map(node, source_path).await;
    let media_type_hint =
        options.and_then(|options| options.format.as_ref().map(Format::media_type));

    sign_export(ExportSigningRequest {
        node,
        codec_name,
        output_path: path,
        source_path,
        source_ranges: source_ranges.as_ref(),
        media_type_hint,
        credential_profile: credential_profile(credentials.profile.clone()),
        info,
    })
    .await?;

    Ok(())
}

/// Sign paths emitted by a codec export without signing the primary output.
pub(crate) async fn sign_encoded_assets(
    node: &Node,
    codec_name: &str,
    options: Option<&EncodeOptions>,
    info: &mut EncodeInfo,
) -> Result<()> {
    let Some(credentials) = options.and_then(|options| options.credentials.as_ref()) else {
        return Ok(());
    };

    let source_path = options.and_then(|options| options.from_path.as_deref());
    let source_ranges = source_range_map(node, source_path).await;

    sign_assets(AssetSigningRequest {
        node,
        codec_name,
        source_path,
        source_ranges: source_ranges.as_ref(),
        credential_profile: credential_profile(credentials.profile.clone()),
        info,
    })
    .await?;

    Ok(())
}

/// Build source ranges keyed by full node id.
///
/// This mirrors the site `nodemap.json` path: encode the document back to its
/// source format, then use a [`PoshMap`] to translate whole-node ranges onto the
/// original source file.
async fn source_range_map(
    node: &Node,
    source_path: Option<&Path>,
) -> Option<BTreeMap<String, SourceRangeSnapshot>> {
    let source_path = source_path?;
    if !source_path.is_file() {
        return None;
    }

    let source_format = Format::from_path(source_path);
    if source_format.is_binary() || source_format.is_lossless() {
        return None;
    }

    let original_source = match read_to_string(source_path).await {
        Ok(source) => source,
        Err(error) => {
            tracing::debug!(
                source = %source_path.display(),
                "Skipping source ranges for Content Credentials: {error}"
            );
            return None;
        }
    };

    let (generated_source, encode_info) = match crate::to_string_with_info(
        node,
        Some(EncodeOptions {
            format: Some(source_format),
            ..Default::default()
        }),
    )
    .await
    {
        Ok(result) => result,
        Err(error) => {
            tracing::debug!(
                source = %source_path.display(),
                "Skipping source ranges for Content Credentials: {error}"
            );
            return None;
        }
    };

    if encode_info.mapping.entries().is_empty() {
        return None;
    }

    let node_ids = encode_info
        .mapping
        .entries()
        .iter()
        .filter(|entry| entry.property.is_none())
        .map(|entry| entry.node_id.clone())
        .collect::<Vec<_>>();

    let poshmap = PoshMap::new(&original_source, &generated_source, encode_info.mapping);
    let mut ranges = BTreeMap::new();

    for node_id in node_ids {
        let Some(range) = poshmap.node_id_to_range8(&node_id) else {
            continue;
        };

        ranges.insert(
            node_id.to_string(),
            SourceRangeSnapshot {
                start_line: range.start.line.saturating_add(1) as u64,
                start_column: range.start.column.saturating_add(1) as u64,
                end_line: range.end.line.saturating_add(1) as u64,
                end_column: range.end.column.saturating_add(1) as u64,
            },
        );
    }

    (!ranges.is_empty()).then_some(ranges)
}

fn credential_profile(profile: CodecCredentialProfile) -> CredentialProfile {
    match profile {
        CodecCredentialProfile::Public => CredentialProfile::Public,
        CodecCredentialProfile::Private => CredentialProfile::Private,
        CodecCredentialProfile::Full => CredentialProfile::Full,
    }
}
