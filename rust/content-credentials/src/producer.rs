//! Sign Stencila assets with a C2PA manifest carrying the
//! `org.stencila.provenance` assertion.

use std::{
    fs::{self, File, Permissions},
    io::{self, Cursor, Write},
    path::{Path, PathBuf},
};

use c2pa::{BoxedSigner, Builder, Context};
use serde::Serialize;
use serde_json::json;
use tempfile::NamedTempFile;

use crate::{
    assertion::asset_kind_for_media_type,
    error::{Error, Result},
    media,
    schema::{PROVENANCE_LABEL, ProvenanceAssertion},
    signer::CredentialSignerConfig,
    snapshot::{AssetSnapshot, ProvenanceSnapshot},
};

/// Whether a manifest is embedded in the asset bytes or written to a sidecar.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ManifestKind {
    Embedded,
    Sidecar,
}

/// Inputs for signing an exported asset.
#[derive(Debug, Clone, Default)]
pub struct SignAssetRequest {
    pub input_path: PathBuf,

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
}

/// Result of a successful signing operation.
#[derive(Debug, Clone)]
pub struct SignedAsset {
    pub asset_path: PathBuf,
    pub manifest_kind: ManifestKind,
    pub sidecar_path: Option<PathBuf>,
    pub assertion_label: &'static str,
    pub assertion_schema: &'static str,
    pub source_digest: String,
    pub signed_asset_digest: String,
    pub media_type: String,
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
            output_path,
            title,
            provenance,
        } = request;

        if !input_path.exists() {
            return Err(Error::InputNotFound(input_path));
        }

        let media_type = media::guess_media_type(&input_path)?;
        let embed = media::embed_supported(&media_type);
        if let Some(output_path) = output_path.as_deref() {
            if !embed && media::sidecar_path(output_path) == output_path {
                return Err(Error::OutputSidecarConflict(output_path.to_path_buf()));
            }
            validate_output_media_type(&input_path, &media_type, output_path)?;
        }

        let source_digest = media::sha256_file(&input_path)?;
        let signer = self.signer.clone();
        let title = title
            .or_else(|| {
                input_path
                    .file_name()
                    .and_then(|n| n.to_str().map(std::string::ToString::to_string))
            })
            .unwrap_or_else(|| "asset".to_string());

        let assertion = provenance.map_or_else(
            || ProvenanceAssertion::new_v1(&media_type, &source_digest),
            |mut snapshot| {
                snapshot.asset =
                    normalize_asset_snapshot(snapshot.asset, &media_type, &source_digest);
                ProvenanceAssertion::from_snapshot(snapshot)
            },
        );

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
                    &signer,
                )
            } else {
                sign_sidecar(
                    &input_path,
                    output_path.as_deref(),
                    &media_for_task,
                    &title,
                    &assertion,
                    &signer,
                )
            }
        })
        .await??;

        let signed_asset_digest = media::sha256_file(&result.0)?;

        Ok(SignedAsset {
            asset_path: result.0,
            manifest_kind: if embed {
                ManifestKind::Embedded
            } else {
                ManifestKind::Sidecar
            },
            sidecar_path: result.1,
            assertion_label: PROVENANCE_LABEL,
            assertion_schema: crate::schema::PROVENANCE_SCHEMA_V1,
            source_digest: source_digest_for_result,
            signed_asset_digest,
            media_type: media_type_for_result,
        })
    }
}

fn normalize_asset_snapshot(
    mut asset: AssetSnapshot,
    media_type: &str,
    source_digest: &str,
) -> AssetSnapshot {
    asset.media_type = media_type.to_string();
    asset.digest = source_digest.to_string();
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
    signer_config: &CredentialSignerConfig,
) -> Result<(PathBuf, Option<PathBuf>)> {
    let dest_path = output_path.map_or_else(|| input_path.to_path_buf(), Path::to_path_buf);

    let parent = dest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    fs::create_dir_all(&parent)?;

    let mut builder = build_builder(media_type, title, assertion)?;
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

/// Sign with a `.c2pa` sidecar next to the asset. Returns (`asset_path`, `Some(sidecar_path)`).
fn sign_sidecar(
    input_path: &Path,
    output_path: Option<&Path>,
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
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

    let mut builder = build_builder(media_type, title, assertion)?;
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
            permissions: &permissions,
        };
        return sign_prehashed_sidecar(&request, signer);
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

    let mut builder =
        build_builder_with_signer(request.media_type, request.title, request.assertion, signer)?;
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
) -> Result<Builder> {
    build_builder_with_context(Context::new(), media_type, title, assertion)
}

fn build_builder_with_signer(
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
    signer: BoxedSigner,
) -> Result<Builder> {
    build_builder_with_context(
        Context::new().with_signer(signer),
        media_type,
        title,
        assertion,
    )
}

fn build_builder_with_context(
    context: Context,
    media_type: &str,
    title: &str,
    assertion: &ProvenanceAssertion,
) -> Result<Builder> {
    let definition = json!({
        "claim_generator_info": [{
            "name": "Stencila",
            "version": stencila_version::STENCILA_VERSION,
        }],
        "title": title,
        "format": media_type,
        "assertions": [
            {
                "label": "c2pa.actions.v2",
                "data": { "actions": [{ "action": "c2pa.created" }] }
            }
        ]
    });

    let mut builder = Builder::from_context(context).with_definition(definition.to_string())?;
    builder.add_assertion(PROVENANCE_LABEL, assertion)?;
    Ok(builder)
}
