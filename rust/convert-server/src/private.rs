//! Private, framed conversion protocol used by trusted deployment workers.

use std::{
    collections::{HashMap, HashSet},
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    Json,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, StatusCode, header::CONTENT_TYPE},
    response::Response,
};
use bytes::Bytes;
use roxmltree::{Document, ParsingOptions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stencila_codecs::{CodecDirection, DecodeOptions, EncodeOptions, Format};
use strum::IntoEnumIterator;
use tempfile::tempdir_in;
use tokio::fs;
use tokio_stream::wrappers::ReceiverStream;
use zip::ZipArchive;

use crate::{AppError, AppState};

const PROTOCOL: &str = "conversion.v1";
const FRAME_MEDIA_TYPE: &str = "application/vnd.stencila.conversion.v1+frames";
const MAX_HEADER_BYTES: usize = 256 * 1024;
const MAX_PAYLOAD_BYTES: usize = 256 * 1024;
const MAX_INPUT_BYTES: usize = 25 * 1024 * 1024;
const MAX_FILES: usize = 257;
const MAX_DIAGNOSTICS: usize = 256;
const SCRATCH_PREFIX: &str = "stencila-convert-private-";

static SCRATCH_ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

#[derive(Clone, Copy)]
enum ConversionDirection {
    Decode,
    Encode,
}

impl ConversionDirection {
    fn codec_direction(self) -> CodecDirection {
        match self {
            Self::Decode => CodecDirection::Decode,
            Self::Encode => CodecDirection::Encode,
        }
    }
}

fn supported_formats(direction: ConversionDirection) -> Vec<Format> {
    Format::iter()
        .filter(|format| {
            !(format.is_unknown() || format.is_other() || matches!(format, Format::Directory))
        })
        .filter(|format| {
            stencila_codecs::get(None, Some(format), Some(direction.codec_direction()))
                .is_ok_and(|codec| codec.is_available())
        })
        .collect()
}

fn protocol_format_name(format: &Format) -> String {
    match format {
        Format::Markdown => "markdown".to_string(),
        _ => format.to_string(),
    }
}

fn parse_protocol_format(value: &str, direction: ConversionDirection) -> Result<Format, AppError> {
    let format = Format::from_name(value);
    if format.is_unknown()
        || format.is_other()
        || matches!(format, Format::Directory)
        || !stencila_codecs::get(None, Some(&format), Some(direction.codec_direction()))
            .is_ok_and(|codec| codec.is_available())
    {
        return Err(client_error(
            "unsupported_format",
            format!("Unsupported private conversion format `{value}`"),
        ));
    }
    Ok(format)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BuildIdentity {
    protocol: &'static str,
    stencila_version: &'static str,
    source_commit: String,
    image_digest: String,
    toolchain: Toolchain,
}

#[derive(Debug, Clone, Serialize)]
struct Toolchain {
    rust: String,
    pandoc: String,
}

impl BuildIdentity {
    pub(super) fn current() -> Self {
        Self {
            protocol: PROTOCOL,
            stencila_version: stencila_version::STENCILA_VERSION,
            source_commit: option_env!("STENCILA_CONVERT_SOURCE_COMMIT")
                .unwrap_or("development")
                .to_string(),
            image_digest: std::env::var("STENCILA_CONVERT_IMAGE_DIGEST")
                .unwrap_or_else(|_| "development".to_string()),
            toolchain: Toolchain {
                rust: option_env!("STENCILA_CONVERT_RUST_VERSION")
                    .unwrap_or("unknown")
                    .to_string(),
                pandoc: pandoc_version(),
            },
        }
    }
}

fn pandoc_version() -> String {
    std::process::Command::new("pandoc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| output.lines().next().map(str::to_string))
        .and_then(|line| line.split_whitespace().nth(1).map(str::to_string))
        .unwrap_or_else(|| "unavailable".to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PrivateHealth {
    ready: bool,
    build: BuildIdentity,
    supported_pairs: Vec<FormatPair>,
    limits: HealthLimits,
    in_flight: usize,
}

#[derive(Serialize)]
struct FormatPair {
    from: String,
    to: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthLimits {
    conversion_timeout_seconds: u64,
    max_upload_bytes: usize,
    max_concurrency: Option<usize>,
    artifact_cache_enabled: bool,
}

pub(super) async fn health(State(state): State<AppState>) -> Json<PrivateHealth> {
    let output_formats = supported_formats(ConversionDirection::Encode);
    let supported_pairs = supported_formats(ConversionDirection::Decode)
        .into_iter()
        .flat_map(|from| {
            output_formats.iter().map(move |to| FormatPair {
                from: protocol_format_name(&from),
                to: protocol_format_name(to),
            })
        })
        .collect();
    Json(PrivateHealth {
        ready: scratch_root().is_ok(),
        build: BuildIdentity::current(),
        supported_pairs,
        limits: HealthLimits {
            conversion_timeout_seconds: state.config.conversion_timeout.as_secs(),
            max_upload_bytes: state.config.max_upload_bytes,
            max_concurrency: state.config.max_concurrency,
            artifact_cache_enabled: state.config.artifacts_enabled(),
        },
        in_flight: state.in_flight(),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestHeader {
    kind: String,
    protocol: String,
    expected_image_digest: Option<String>,
    job_id: String,
    attempt: u32,
    from: String,
    to: String,
    options: RequestOptions,
    main_path: String,
    manuscript_path: Option<String>,
    files: Vec<FileDeclaration>,
    policy: ProcessingPolicy,
    bounds: RequestBounds,
}

#[derive(Debug, Deserialize)]
struct RequestOptions {
    compact: bool,
    standalone: bool,
    media: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProcessingPolicy {
    allow_models: bool,
    allow_code_execution: bool,
    allow_network: bool,
    supplements: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestBounds {
    max_expanded_bytes: usize,
    max_archive_entries: usize,
    max_artifact_bytes: usize,
    max_result_bytes: usize,
    max_artifacts: usize,
}

#[derive(Debug, Deserialize)]
struct FileDeclaration {
    path: String,
    size: usize,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct FileFrame {
    path: String,
    offset: usize,
    bytes: usize,
}

#[derive(Debug, Deserialize)]
struct KindFrame {
    kind: String,
}

struct Bundle {
    header: RequestHeader,
    files: HashMap<String, Vec<u8>>,
}

#[derive(Serialize)]
struct ResultHeader<'a> {
    kind: &'static str,
    protocol: &'static str,
    build: BuildIdentity,
    #[serde(rename = "jobId")]
    job_id: &'a str,
    #[serde(rename = "artifactCount")]
    artifact_count: usize,
    #[serde(rename = "totalBytes")]
    total_bytes: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactHeader<'a> {
    kind: &'static str,
    id: &'a str,
    role: &'a str,
    path: &'a str,
    media_type: &'a str,
    size: usize,
    sha256: String,
    media: Option<MediaInfo>,
}

#[derive(Serialize)]
struct MediaInfo {
    transformations: Vec<MediaTransformation>,
}

#[derive(Serialize)]
struct MediaTransformation {
    kind: String,
    detail: String,
}

#[derive(Serialize)]
struct PartHeader<'a> {
    kind: &'static str,
    id: &'a str,
    offset: usize,
    bytes: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Trailer {
    kind: &'static str,
    status: &'static str,
    error: Option<TrailerError>,
    decode: Option<DiagnosticReport>,
    encode: Option<DiagnosticReport>,
    missing_assets: Vec<MissingAsset>,
    missing_assets_truncated: bool,
    supplements: Vec<SupplementOutcome>,
    supplements_truncated: bool,
    external_dependencies: Vec<String>,
    codecs: CodecNames,
    model_calls: usize,
    usage: Usage,
}

#[derive(Serialize)]
struct TrailerError {
    code: String,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticReport {
    messages: Vec<DiagnosticMessage>,
    messages_truncated: bool,
    losses: Vec<DiagnosticLoss>,
    losses_truncated: bool,
}

#[derive(Serialize)]
struct DiagnosticMessage {
    level: String,
    message: String,
    location: Option<String>,
}

#[derive(Serialize)]
struct DiagnosticLoss {
    #[serde(rename = "type")]
    kind: String,
    count: usize,
}

#[derive(Serialize)]
struct MissingAsset {
    path: String,
    reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SupplementOutcome {
    path: String,
    outcome: String,
    artifact_id: Option<String>,
}

#[derive(Serialize)]
struct CodecNames {
    decode: String,
    encode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Usage {
    duration_ms: usize,
    input_bytes: usize,
    output_bytes: usize,
}

struct Artifact {
    id: String,
    role: &'static str,
    path: String,
    media_type: String,
    bytes: Vec<u8>,
}

struct ConversionResult {
    artifacts: Vec<Artifact>,
    decode: DiagnosticReport,
    encode: DiagnosticReport,
    missing_assets: Vec<MissingAsset>,
    supplements: Vec<SupplementOutcome>,
    codecs: CodecNames,
}

struct ResponseUsage {
    input_bytes: usize,
    output_bytes: usize,
    duration_ms: usize,
    delivery_timeout: Duration,
}

/// A process-level deadline for synchronous codec and archive work.
///
/// Tokio's timeout cannot fire while a codec blocks its executor thread. The
/// converter runs one job per container, so terminating the process is the
/// only reliable way to ensure timed-out native work cannot outlive its
/// scratch directory or admit another job.
struct HardDeadline(Arc<(Mutex<bool>, Condvar)>);

impl HardDeadline {
    fn start(duration: Duration) -> Self {
        let state = Arc::new((Mutex::new(false), Condvar::new()));
        let watcher = Arc::clone(&state);
        std::thread::spawn(move || {
            let (complete, changed) = &*watcher;
            let guard = complete.lock().unwrap_or_else(|error| error.into_inner());
            let result = changed
                .wait_timeout_while(guard, duration, |complete| !*complete)
                .unwrap_or_else(|error| error.into_inner());
            if result.1.timed_out() && !*result.0 {
                tracing::error!("Private conversion exceeded its hard deadline");
                std::process::exit(124);
            }
        });
        Self(state)
    }
}

impl Drop for HardDeadline {
    fn drop(&mut self) {
        let (complete, changed) = &*self.0;
        let mut complete = complete.lock().unwrap_or_else(|error| error.into_inner());
        *complete = true;
        changed.notify_one();
    }
}

pub(super) async fn convert(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Response, AppError> {
    if request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(FRAME_MEDIA_TYPE)
    {
        return Err(client_error(
            "invalid_media_type",
            "Use the conversion.v1 framed media type",
        ));
    }

    let permit = state.acquire().await?;
    let in_flight = state.begin_conversion();
    let started = Instant::now();
    let bytes = tokio::time::timeout(
        state.config.conversion_timeout,
        to_bytes(request.into_body(), state.config.max_request_bytes()),
    )
    .await
    .map_err(|_| client_error("upload_timeout", "Private upload exceeded its deadline"))?
    .map_err(|_| {
        client_error(
            "request_too_large",
            "The framed request exceeds the configured limit",
        )
    })?;
    let bundle = parse_bundle(&bytes)?;
    let formats = validate_header(&bundle.header, &state)?;

    let input_bytes = bundle.files.values().map(Vec::len).sum();
    let remaining = state
        .config
        .conversion_timeout
        .saturating_sub(started.elapsed());
    if remaining.is_zero() {
        return Err(client_error(
            "conversion_timeout",
            "Private conversion exceeded its deadline",
        ));
    }
    let hard_deadline = HardDeadline::start(remaining);
    let outcome = match tokio::time::timeout(remaining, convert_bundle(&bundle, &formats)).await {
        Ok(outcome) => outcome,
        Err(_) => Err(client_error(
            "conversion_timeout",
            "Private conversion exceeded its deadline",
        )),
    };
    drop(hard_deadline);

    let result = match outcome {
        Ok(result) => result,
        Err(error) => {
            let body = failure_response_body(
                &bundle.header,
                &error,
                ResponseUsage {
                    input_bytes,
                    output_bytes: 0,
                    duration_ms: started.elapsed().as_millis() as usize,
                    delivery_timeout: state
                        .config
                        .conversion_timeout
                        .saturating_sub(started.elapsed()),
                },
                permit,
                in_flight,
            );
            return Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, FRAME_MEDIA_TYPE)
                .body(body)
                .map_err(AppError::internal);
        }
    };

    let output_bytes = result
        .artifacts
        .iter()
        .map(|artifact| artifact.bytes.len())
        .sum();
    let body = response_body(
        &bundle.header,
        result,
        ResponseUsage {
            input_bytes,
            output_bytes,
            duration_ms: started.elapsed().as_millis() as usize,
            delivery_timeout: state
                .config
                .conversion_timeout
                .saturating_sub(started.elapsed()),
        },
        permit,
        in_flight,
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, FRAME_MEDIA_TYPE)
        .body(body)
        .map_err(AppError::internal)
}

fn parse_bundle(bytes: &[u8]) -> Result<Bundle, AppError> {
    let mut cursor = 0;
    let (header_bytes, payload) = next_frame(bytes, &mut cursor, false)?;
    if !payload.is_empty() {
        return Err(client_error(
            "invalid_frame",
            "The request header cannot carry a payload",
        ));
    }
    let header: RequestHeader = serde_json::from_slice(header_bytes).map_err(|_| {
        client_error(
            "invalid_header",
            "The first frame is not a valid request header",
        )
    })?;
    if header.kind != "request" {
        return Err(client_error(
            "invalid_header",
            "The first frame must be a request",
        ));
    }
    if header.files.is_empty() || header.files.len() > MAX_FILES {
        return Err(client_error(
            "invalid_bundle",
            "The bundle file count is out of range",
        ));
    }

    let declared: HashMap<_, _> = header
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    if declared.len() != header.files.len() {
        return Err(client_error(
            "invalid_bundle",
            "Bundle paths must be unique",
        ));
    }
    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    let mut ended = false;
    while cursor < bytes.len() {
        let (json, payload) = next_frame(bytes, &mut cursor, true)?;
        let kind: KindFrame = serde_json::from_slice(json)
            .map_err(|_| client_error("invalid_frame", "A request frame is not valid JSON"))?;
        match kind.kind.as_str() {
            "file" => {
                if ended {
                    return Err(client_error(
                        "invalid_frame",
                        "A file frame followed the end frame",
                    ));
                }
                let frame: FileFrame = serde_json::from_slice(json)
                    .map_err(|_| client_error("invalid_frame", "A file frame is malformed"))?;
                let declaration = declared.get(frame.path.as_str()).ok_or_else(|| {
                    client_error("invalid_bundle", "A file frame names an undeclared path")
                })?;
                if frame.bytes != payload.len() {
                    return Err(client_error(
                        "invalid_frame",
                        "A file frame payload length differs",
                    ));
                }
                let file = files.entry(frame.path).or_default();
                if frame.offset != file.len()
                    || file.len().saturating_add(payload.len()) > declaration.size
                {
                    return Err(client_error(
                        "invalid_bundle",
                        "File parts are out of order or oversized",
                    ));
                }
                file.extend_from_slice(payload);
            }
            "end" => {
                if !payload.is_empty() || ended {
                    return Err(client_error("invalid_frame", "The end frame is malformed"));
                }
                ended = true;
            }
            _ => return Err(client_error("invalid_frame", "Unknown request frame kind")),
        }
    }
    if !ended {
        return Err(client_error(
            "invalid_frame",
            "The request has no end frame",
        ));
    }

    let total: usize = header.files.iter().map(|file| file.size).sum();
    if total > MAX_INPUT_BYTES {
        return Err(client_error(
            "input_too_large",
            "The input bundle exceeds 25 MiB",
        ));
    }
    for declaration in &header.files {
        validate_relative_path(&declaration.path)?;
        let content = files
            .get(&declaration.path)
            .ok_or_else(|| client_error("invalid_bundle", "A declared file has no payload"))?;
        if content.len() != declaration.size || hex_sha256(content) != declaration.sha256 {
            return Err(client_error(
                "input_mismatch",
                "An input size or SHA-256 does not match",
            ));
        }
    }
    Ok(Bundle { header, files })
}

fn next_frame<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    payload_allowed: bool,
) -> Result<(&'a [u8], &'a [u8]), AppError> {
    let length = bytes
        .get(*cursor..cursor.saturating_add(4))
        .and_then(|value| value.try_into().ok())
        .map(u32::from_be_bytes)
        .map(|value| value as usize)
        .ok_or_else(|| client_error("invalid_frame", "The stream ended before a frame header"))?;
    if length == 0 || length > MAX_HEADER_BYTES {
        return Err(client_error(
            "invalid_frame",
            "A frame header length is out of range",
        ));
    }
    *cursor += 4;
    let json = bytes
        .get(*cursor..cursor.saturating_add(length))
        .ok_or_else(|| client_error("invalid_frame", "The stream ended within a frame header"))?;
    *cursor += length;
    let payload_length = if payload_allowed {
        serde_json::from_slice::<serde_json::Value>(json)
            .ok()
            .and_then(|value| value.get("bytes").and_then(serde_json::Value::as_u64))
            .unwrap_or(0) as usize
    } else {
        0
    };
    if payload_length > MAX_PAYLOAD_BYTES {
        return Err(client_error(
            "invalid_frame",
            "A frame payload is too large",
        ));
    }
    let payload = bytes
        .get(*cursor..cursor.saturating_add(payload_length))
        .ok_or_else(|| client_error("invalid_frame", "The stream ended within a frame payload"))?;
    *cursor += payload_length;
    Ok((json, payload))
}

struct ConversionFormats {
    from: Format,
    to: Format,
}

fn validate_header(
    header: &RequestHeader,
    state: &AppState,
) -> Result<ConversionFormats, AppError> {
    if header.protocol != PROTOCOL {
        return Err(client_error(
            "protocol_mismatch",
            "Unsupported private protocol version",
        ));
    }
    let build = BuildIdentity::current();
    if header
        .expected_image_digest
        .as_ref()
        .is_some_and(|expected| expected != &build.image_digest)
    {
        return Err(client_error(
            "build_mismatch",
            "The expected image is not running",
        ));
    }
    if header.job_id.is_empty() || header.job_id.len() > 128 || header.attempt == 0 {
        return Err(client_error(
            "invalid_request",
            "The job identity is invalid",
        ));
    }
    let from = parse_protocol_format(&header.from, ConversionDirection::Decode)?;
    let to = parse_protocol_format(&header.to, ConversionDirection::Encode)?;
    if header.policy.allow_models
        || header.policy.allow_code_execution
        || header.policy.allow_network
        || header.policy.supplements != "retain-original"
    {
        return Err(client_error(
            "policy_refused",
            "Only the no-model, no-code, no-network policy is admitted",
        ));
    }
    if header.options.media != "sidecar" {
        return Err(client_error(
            "invalid_options",
            "Private conversion requires sidecar media",
        ));
    }
    validate_relative_path(&header.main_path)?;
    if !header
        .files
        .iter()
        .any(|file| file.path == header.main_path)
    {
        return Err(client_error(
            "invalid_bundle",
            "The main path is not in the bundle",
        ));
    }
    if let Some(path) = &header.manuscript_path {
        validate_relative_path(path)?;
    }
    if header.bounds.max_expanded_bytes == 0
        || header.bounds.max_expanded_bytes > 100 * 1024 * 1024
        || header.bounds.max_archive_entries == 0
        || header.bounds.max_archive_entries > 1_000
        || header.bounds.max_artifact_bytes == 0
        || header.bounds.max_artifact_bytes > 20 * 1024 * 1024
        || header.bounds.max_result_bytes == 0
        || header.bounds.max_result_bytes > 100 * 1024 * 1024
        || header.bounds.max_artifacts == 0
        || header.bounds.max_artifacts > 256
        || state.config.artifacts_enabled()
    {
        return Err(client_error(
            "bounds_refused",
            "Requested bounds or server cache settings are unsafe",
        ));
    }
    Ok(ConversionFormats { from, to })
}

async fn convert_bundle(
    bundle: &Bundle,
    formats: &ConversionFormats,
) -> Result<ConversionResult, AppError> {
    let scratch = scratch_root().map_err(AppError::internal)?;
    let temp = tempdir_in(scratch).map_err(AppError::internal)?;
    let source_root = temp.path().join("source");
    let output_root = temp.path().join("result");
    fs::create_dir_all(&source_root)
        .await
        .map_err(AppError::internal)?;
    fs::create_dir_all(&output_root)
        .await
        .map_err(AppError::internal)?;

    for (path, bytes) in &bundle.files {
        let destination = confined_join(&source_root, path)?;
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(AppError::internal)?;
        }
        fs::write(destination, bytes)
            .await
            .map_err(AppError::internal)?;
    }

    let mut artifacts = Vec::new();
    let mut supplements = Vec::new();
    let (decode_path, decode_format, reference_root) = if formats.from == Format::Meca {
        let archive = bundle
            .files
            .get(&bundle.header.main_path)
            .ok_or_else(|| client_error("invalid_bundle", "The MECA archive is unavailable"))?;
        let extracted = temp.path().join("meca");
        std::fs::create_dir_all(&extracted).map_err(AppError::internal)?;
        let entries = extract_meca(archive, &extracted, &bundle.header)?;
        let manifest = read_meca_manifest(&extracted, bundle.header.manuscript_path.as_deref())?;
        for path in entries.into_iter().filter(|path| {
            path != &manifest.manuscript
                && path != "manifest.xml"
                && path != "mimetype"
                && path != "directives.xml"
                && path != "transfer.xml"
        }) {
            let bytes =
                std::fs::read(confined_join(&extracted, &path)?).map_err(AppError::internal)?;
            let role = manifest.roles.get(&path).copied().unwrap_or("original");
            let id = format!("{role}-{:04}", artifacts.len() + 1);
            if role == "supplement" {
                supplements.push(SupplementOutcome {
                    path: path.clone(),
                    outcome: "retained-original".to_string(),
                    artifact_id: Some(id.clone()),
                });
            }
            artifacts.push(Artifact {
                id,
                role,
                media_type: media_type(&path),
                path,
                bytes,
            });
        }
        artifacts.push(Artifact {
            id: "original".to_string(),
            role: "original",
            path: bundle.header.main_path.clone(),
            media_type: "application/zip".to_string(),
            bytes: archive.clone(),
        });
        (
            confined_join(&extracted, &manifest.manuscript)?,
            Format::Jats,
            extracted,
        )
    } else {
        (
            confined_join(&source_root, &bundle.header.main_path)?,
            formats.from.clone(),
            source_root.clone(),
        )
    };

    let decode_options = Some(DecodeOptions {
        format: Some(decode_format),
        reproducible: Some(false),
        ignore_artifacts: Some(true),
        no_artifacts: Some(true),
        embed_media: Some(false),
        embed_supplements: Some(false),
        structuring_options: Default::default(),
        ..Default::default()
    });
    let (node, _, decode_info) = stencila_codecs::from_path_with_info(&decode_path, decode_options)
        .await
        .map_err(AppError::conversion)?;
    let mut node_value = serde_json::to_value(node).map_err(AppError::internal)?;
    let mut missing_assets = Vec::new();
    let mut seen_missing = HashSet::new();
    normalize_media_objects(
        &mut node_value,
        &decode_path,
        &reference_root,
        &mut missing_assets,
        &mut seen_missing,
    )?;
    let node = serde_json::from_value(node_value).map_err(AppError::internal)?;

    let target_format = formats.to.clone();
    let primary_path = format!("document.{}", target_format.extension());
    let output_path = output_root.join(&primary_path);
    let media_dir = output_root.join("document.media");
    let encode_options = Some(EncodeOptions {
        format: Some(target_format.clone()),
        compact: Some(bundle.header.options.compact),
        standalone: Some(bundle.header.options.standalone),
        reproducible: Some(false),
        from_path: Some(reference_root.join("document")),
        extract_media: Some(media_dir.clone()),
        embed_media: Some(false),
        recurse: Some(false),
        ..Default::default()
    });
    let encode_info = stencila_codecs::to_path_with_info(&node, &output_path, encode_options)
        .await
        .map_err(AppError::conversion)?;
    let primary = fs::read(&output_path).await.map_err(AppError::internal)?;
    artifacts.insert(
        0,
        Artifact {
            id: "primary".to_string(),
            role: "primary",
            path: primary_path,
            media_type: target_format.media_type(),
            bytes: primary,
        },
    );

    for (index, declaration) in bundle.header.files.iter().enumerate() {
        if declaration.path == bundle.header.main_path || formats.from == Format::Meca {
            continue;
        }
        artifacts.push(Artifact {
            id: format!("asset-{:04}", index + 1),
            role: "asset",
            path: declaration.path.clone(),
            media_type: media_type(&declaration.path),
            bytes: bundle.files[&declaration.path].clone(),
        });
    }
    if media_dir.exists() {
        collect_output_files(&media_dir, &output_root, &mut artifacts)?;
    }

    enforce_result_bounds(&artifacts, &bundle.header.bounds)?;
    Ok(ConversionResult {
        artifacts,
        decode: diagnostic_report(&decode_info.messages, &decode_info.losses),
        encode: DiagnosticReport {
            messages: Vec::new(),
            messages_truncated: false,
            losses: encode_info
                .losses
                .iter()
                .take(MAX_DIAGNOSTICS)
                .map(|(kind, count)| DiagnosticLoss {
                    kind: kind.to_string(),
                    count,
                })
                .collect(),
            losses_truncated: encode_info.losses.iter().count() > MAX_DIAGNOSTICS,
        },
        missing_assets,
        supplements,
        codecs: CodecNames {
            decode: bundle.header.from.clone(),
            encode: bundle.header.to.clone(),
        },
    })
}

fn extract_meca(
    archive: &[u8],
    root: &Path,
    header: &RequestHeader,
) -> Result<Vec<String>, AppError> {
    let mut zip = ZipArchive::new(Cursor::new(archive))
        .map_err(|_| client_error("invalid_meca", "The MECA input is not a ZIP archive"))?;
    if zip.len() > header.bounds.max_archive_entries {
        return Err(client_error(
            "archive_limit",
            "The MECA archive has too many entries",
        ));
    }
    let mut total = 0usize;
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    let mut files: HashSet<String> = HashSet::new();
    for index in 0..zip.len() {
        let entry = zip
            .by_index(index)
            .map_err(|_| client_error("invalid_meca", "A MECA entry could not be read"))?;
        let path = entry.name().to_string();
        validate_relative_path(&path)?;
        let normalized = path.trim_end_matches('/');
        if normalized.is_empty() || !seen.insert(normalized.to_string()) {
            return Err(client_error("invalid_meca", "MECA paths must be unique"));
        }
        if entry.is_dir() {
            if files.contains(normalized) {
                return Err(client_error(
                    "invalid_meca",
                    "A MECA path is both a file and directory",
                ));
            }
            std::fs::create_dir_all(confined_join(root, &path)?).map_err(AppError::internal)?;
            continue;
        }
        if files.iter().any(|other| {
            other.starts_with(&format!("{normalized}/"))
                || normalized.starts_with(&format!("{other}/"))
        }) {
            return Err(client_error(
                "invalid_meca",
                "MECA file paths conflict with directories",
            ));
        }
        files.insert(normalized.to_string());
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(client_error(
                "invalid_meca",
                "MECA symlinks are not permitted",
            ));
        }
        let size = usize::try_from(entry.size())
            .map_err(|_| client_error("archive_limit", "A MECA entry is too large"))?;
        total = total
            .checked_add(size)
            .ok_or_else(|| client_error("archive_limit", "MECA expanded size overflowed"))?;
        if total > header.bounds.max_expanded_bytes {
            return Err(client_error(
                "archive_limit",
                "The MECA expanded size exceeds its bound",
            ));
        }
        let destination = confined_join(root, &path)?;
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::internal)?;
        }
        let mut output = std::fs::File::create(destination).map_err(AppError::internal)?;
        let copied = std::io::copy(
            &mut entry.take((size as u64).saturating_add(1)),
            &mut output,
        )
        .map_err(AppError::internal)?;
        if copied != size as u64 {
            return Err(client_error(
                "archive_limit",
                "A MECA entry's expanded size differs from its declaration",
            ));
        }
        paths.push(path);
    }
    Ok(paths)
}

struct MecaManifest {
    manuscript: String,
    roles: HashMap<String, &'static str>,
}

fn read_meca_manifest(root: &Path, selected: Option<&str>) -> Result<MecaManifest, AppError> {
    let manifest = std::fs::read_to_string(root.join("manifest.xml"))
        .map_err(|_| client_error("invalid_meca", "MECA manifest.xml is missing"))?;
    let document = Document::parse_with_options(
        &manifest,
        ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .map_err(|_| client_error("invalid_meca", "MECA manifest.xml is malformed"))?;
    let article_xml: Vec<_> = document
        .descendants()
        .filter(|node| node.has_tag_name("item") && node.attribute("type") == Some("article"))
        .flat_map(|item| item.children())
        .filter(|node| {
            node.has_tag_name("instance") && node.attribute("media-type") == Some("application/xml")
        })
        .filter_map(|node| node.attribute("href"))
        .collect();
    if article_xml.len() != 1 || selected.is_some_and(|selected| selected != article_xml[0]) {
        return Err(client_error(
            "invalid_meca",
            "The selected manuscript does not uniquely match the MECA manifest",
        ));
    }
    let manuscript = article_xml[0].to_string();
    validate_relative_path(&manuscript)?;
    if !confined_join(root, &manuscript)?.is_file() {
        return Err(client_error(
            "invalid_meca",
            "The selected MECA manuscript is missing",
        ));
    }

    let mut roles = HashMap::new();
    for item in document
        .descendants()
        .filter(|node| node.has_tag_name("item"))
    {
        let role = match item.attribute("type") {
            Some("figure" | "media") => "asset",
            Some("supplement" | "supplementary-material") => "supplement",
            _ => "original",
        };
        for path in item
            .children()
            .filter(|node| node.has_tag_name("instance"))
            .filter_map(|node| node.attribute("href"))
        {
            validate_relative_path(path)?;
            roles.insert(path.to_string(), role);
        }
    }
    Ok(MecaManifest { manuscript, roles })
}

fn normalize_media_objects(
    value: &mut serde_json::Value,
    document_path: &Path,
    root: &Path,
    missing: &mut Vec<MissingAsset>,
    seen_missing: &mut HashSet<String>,
) -> Result<(), AppError> {
    match value {
        serde_json::Value::Object(object) => {
            if object
                .get("type")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|kind| matches!(kind, "ImageObject" | "AudioObject" | "VideoObject"))
                && let Some(reference) = object
                    .get("contentUrl")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                && let Some(normalized) = normalize_media_reference(
                    &reference,
                    document_path,
                    root,
                    missing,
                    seen_missing,
                )?
            {
                object.insert(
                    "contentUrl".to_string(),
                    serde_json::Value::String(normalized),
                );
            }
            for value in object.values_mut() {
                normalize_media_objects(value, document_path, root, missing, seen_missing)?;
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                normalize_media_objects(value, document_path, root, missing, seen_missing)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn normalize_media_reference(
    reference: &str,
    document_path: &Path,
    root: &Path,
    missing: &mut Vec<MissingAsset>,
    seen_missing: &mut HashSet<String>,
) -> Result<Option<String>, AppError> {
    if reference.is_empty() || reference.starts_with('#') {
        return Ok(None);
    }
    if reference.starts_with("data:")
        || reference.starts_with("file:")
        || reference.contains("://")
        || reference.starts_with("//")
    {
        return Err(client_error(
            "unsafe_reference",
            "Document media references must name supplied relative files",
        ));
    }
    validate_relative_path(reference)?;
    let parent = document_path.parent().unwrap_or(root);
    let resolved = parent.join(reference);
    let relative = resolved.strip_prefix(root).map_err(|_| {
        client_error(
            "unsafe_reference",
            "A document media reference escapes the source root",
        )
    })?;
    let normalized = relative.to_string_lossy().replace('\\', "/");
    if !resolved.is_file() && seen_missing.insert(normalized.clone()) {
        missing.push(MissingAsset {
            path: normalized.clone(),
            reason: "not supplied in the source bundle".to_string(),
        });
    }
    Ok(Some(normalized))
}

fn collect_output_files(
    root: &Path,
    output_root: &Path,
    artifacts: &mut Vec<Artifact>,
) -> Result<(), AppError> {
    for entry in std::fs::read_dir(root).map_err(AppError::internal)? {
        let entry = entry.map_err(AppError::internal)?;
        if entry.file_type().map_err(AppError::internal)?.is_dir() {
            collect_output_files(&entry.path(), output_root, artifacts)?;
        } else {
            let relative = entry
                .path()
                .strip_prefix(output_root)
                .map_err(AppError::internal)?
                .to_string_lossy()
                .replace('\\', "/");
            artifacts.push(Artifact {
                id: format!("derived-{:04}", artifacts.len() + 1),
                role: "asset",
                media_type: media_type(&relative),
                path: relative,
                bytes: std::fs::read(entry.path()).map_err(AppError::internal)?,
            });
        }
    }
    Ok(())
}

fn enforce_result_bounds(artifacts: &[Artifact], bounds: &RequestBounds) -> Result<(), AppError> {
    if artifacts.len() > bounds.max_artifacts {
        return Err(client_error(
            "result_limit",
            "The conversion produced too many artifacts",
        ));
    }
    let mut total = 0usize;
    let mut ids = HashSet::new();
    let mut paths = HashSet::new();
    for artifact in artifacts {
        if !ids.insert(&artifact.id) || !paths.insert(&artifact.path) {
            return Err(client_error(
                "result_limit",
                "Conversion artifacts must have unique ids and paths",
            ));
        }
        if artifact.bytes.len() > bounds.max_artifact_bytes {
            return Err(client_error(
                "result_limit",
                "A conversion artifact exceeds its bound",
            ));
        }
        total = total.saturating_add(artifact.bytes.len());
    }
    if total > bounds.max_result_bytes {
        return Err(client_error(
            "result_limit",
            "The conversion result exceeds its bound",
        ));
    }
    Ok(())
}

fn response_body(
    header: &RequestHeader,
    result: ConversionResult,
    usage: ResponseUsage,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    in_flight: crate::InFlightGuard,
) -> Body {
    let (sender, receiver) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(2);
    let job_id = header.job_id.clone();
    tokio::spawn(async move {
        // The concurrency permit covers delivery as well as codec work. A slow or abandoned
        // consumer therefore cannot overlap another conversion in a single-concurrency image.
        let _permit = permit;
        let _in_flight = in_flight;
        let delivery = async move {
            let send = async |frame: Result<Bytes, AppError>| {
                let frame = frame.map_err(std::io::Error::other);
                sender.send(frame).await.is_ok()
            };
            if !send(frame(
                &ResultHeader {
                    kind: "result",
                    protocol: PROTOCOL,
                    build: BuildIdentity::current(),
                    job_id: &job_id,
                    artifact_count: result.artifacts.len(),
                    total_bytes: usage.output_bytes,
                },
                &[],
            ))
            .await
            {
                return;
            }
            for artifact in result.artifacts {
                if !send(frame(
                    &ArtifactHeader {
                        kind: "artifact",
                        id: &artifact.id,
                        role: artifact.role,
                        path: &artifact.path,
                        media_type: &artifact.media_type,
                        size: artifact.bytes.len(),
                        sha256: hex_sha256(&artifact.bytes),
                        media: Some(MediaInfo {
                            transformations: Vec::new(),
                        }),
                    },
                    &[],
                ))
                .await
                {
                    return;
                }
                for (index, chunk) in artifact.bytes.chunks(MAX_PAYLOAD_BYTES).enumerate() {
                    if !send(frame(
                        &PartHeader {
                            kind: "part",
                            id: &artifact.id,
                            offset: index * MAX_PAYLOAD_BYTES,
                            bytes: chunk.len(),
                        },
                        chunk,
                    ))
                    .await
                    {
                        return;
                    }
                }
            }
            let _ = send(trailer_frame(Trailer {
                kind: "trailer",
                status: "completed",
                error: None,
                decode: Some(result.decode),
                encode: Some(result.encode),
                missing_assets: result.missing_assets,
                missing_assets_truncated: false,
                supplements: result.supplements,
                supplements_truncated: false,
                external_dependencies: Vec::new(),
                codecs: result.codecs,
                model_calls: 0,
                usage: Usage {
                    duration_ms: usage.duration_ms,
                    input_bytes: usage.input_bytes,
                    output_bytes: usage.output_bytes,
                },
            }))
            .await;
        };
        if tokio::time::timeout(usage.delivery_timeout, delivery)
            .await
            .is_err()
        {
            tracing::error!("Private conversion result delivery exceeded its deadline");
        }
    });
    Body::from_stream(ReceiverStream::new(receiver))
}

fn failure_response_body(
    header: &RequestHeader,
    error: &AppError,
    usage: ResponseUsage,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    in_flight: crate::InFlightGuard,
) -> Body {
    let (sender, receiver) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(2);
    let job_id = header.job_id.clone();
    let from = header.from.clone();
    let to = header.to.clone();
    let code = error.code().to_string();
    let message = error.message().chars().take(1_024).collect::<String>();
    tokio::spawn(async move {
        let _permit = permit;
        let _in_flight = in_flight;
        let delivery = async move {
            let headers = frame(
                &ResultHeader {
                    kind: "result",
                    protocol: PROTOCOL,
                    build: BuildIdentity::current(),
                    job_id: &job_id,
                    artifact_count: 0,
                    total_bytes: 0,
                },
                &[],
            );
            let trailer = trailer_frame(Trailer {
                kind: "trailer",
                status: "failed",
                error: Some(TrailerError { code, message }),
                decode: None,
                encode: None,
                missing_assets: Vec::new(),
                missing_assets_truncated: false,
                supplements: Vec::new(),
                supplements_truncated: false,
                external_dependencies: Vec::new(),
                codecs: CodecNames {
                    decode: from,
                    encode: to,
                },
                model_calls: 0,
                usage: Usage {
                    duration_ms: usage.duration_ms,
                    input_bytes: usage.input_bytes,
                    output_bytes: 0,
                },
            });
            for value in [headers, trailer] {
                if sender
                    .send(value.map_err(std::io::Error::other))
                    .await
                    .is_err()
                {
                    return;
                }
            }
        };
        if tokio::time::timeout(usage.delivery_timeout, delivery)
            .await
            .is_err()
        {
            tracing::error!("Private conversion failure delivery exceeded its deadline");
        }
    });
    Body::from_stream(ReceiverStream::new(receiver))
}

/// Prepare one process-owned scratch root and reclaim roots abandoned by a
/// previous hard-deadline exit before the server reports readiness.
pub(super) fn prepare_scratch_root() -> Result<(), std::io::Error> {
    scratch_root().map(|_| ())
}

fn scratch_root() -> Result<&'static Path, std::io::Error> {
    if let Some(root) = SCRATCH_ROOT.get() {
        std::fs::create_dir_all(root)?;
        return Ok(root);
    }
    let parent = std::env::temp_dir();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let current = parent.join(format!("{SCRATCH_PREFIX}{}-{nonce}", std::process::id()));
    reclaim_scratch_roots(&parent, &current)?;
    std::fs::create_dir_all(&current)?;
    let _ = SCRATCH_ROOT.set(current);
    SCRATCH_ROOT
        .get()
        .map(PathBuf::as_path)
        .ok_or_else(|| std::io::Error::other("private scratch root was not initialized"))
}

fn reclaim_scratch_roots(parent: &Path, current: &Path) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(parent)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        let owner_is_running = name
            .strip_prefix(SCRATCH_PREFIX)
            .and_then(|value| value.split('-').next())
            .and_then(|value| value.parse::<u32>().ok())
            .is_some_and(|pid| {
                pid != std::process::id() && Path::new("/proc").join(pid.to_string()).exists()
            });
        let is_dir = match entry.file_type() {
            Ok(kind) => kind.is_dir(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if entry.path() != current
            && is_dir
            && name.starts_with(SCRATCH_PREFIX)
            && !owner_is_running
            && let Err(error) = std::fs::remove_dir_all(entry.path())
            && error.kind() != std::io::ErrorKind::NotFound
        {
            return Err(error);
        }
    }
    Ok(())
}

fn trailer_frame(mut trailer: Trailer) -> Result<Bytes, AppError> {
    loop {
        let json = serde_json::to_vec(&trailer).map_err(AppError::internal)?;
        if json.len() <= MAX_HEADER_BYTES {
            return frame(&trailer, &[]);
        }
        if trailer.missing_assets.pop().is_some() {
            trailer.missing_assets_truncated = true;
        } else if trailer.supplements.pop().is_some() {
            trailer.supplements_truncated = true;
        } else if let Some(report) = trailer.decode.as_mut()
            && report.messages.pop().is_some()
        {
            report.messages_truncated = true;
        } else if let Some(report) = trailer.encode.as_mut()
            && report.messages.pop().is_some()
        {
            report.messages_truncated = true;
        } else if let Some(report) = trailer.decode.as_mut()
            && report.losses.pop().is_some()
        {
            report.losses_truncated = true;
        } else if let Some(report) = trailer.encode.as_mut()
            && report.losses.pop().is_some()
        {
            report.losses_truncated = true;
        } else if trailer.external_dependencies.pop().is_none() {
            return Err(AppError::internal(
                "A generated conversion trailer exceeded its bound",
            ));
        }
    }
}

fn frame<T: Serialize>(header: &T, payload: &[u8]) -> Result<Bytes, AppError> {
    let json = serde_json::to_vec(header).map_err(AppError::internal)?;
    if json.is_empty() || json.len() > MAX_HEADER_BYTES || payload.len() > MAX_PAYLOAD_BYTES {
        return Err(AppError::internal(
            "A generated conversion frame exceeded its bound",
        ));
    }
    let mut bytes = Vec::with_capacity(4 + json.len() + payload.len());
    bytes.extend_from_slice(&(json.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&json);
    bytes.extend_from_slice(payload);
    Ok(Bytes::from(bytes))
}

fn diagnostic_report(
    messages: &stencila_codecs::Messages,
    losses: &stencila_codecs::Losses,
) -> DiagnosticReport {
    DiagnosticReport {
        messages: messages
            .iter()
            .take(MAX_DIAGNOSTICS)
            .map(|message| DiagnosticMessage {
                level: format!("{:?}", message.level).to_lowercase(),
                message: message.message.clone(),
                location: message.start_line.map(|line| format!("line {}", line + 1)),
            })
            .collect(),
        messages_truncated: messages.len() > MAX_DIAGNOSTICS,
        losses: losses
            .iter()
            .take(MAX_DIAGNOSTICS)
            .map(|(kind, count)| DiagnosticLoss {
                kind: kind.to_string(),
                count,
            })
            .collect(),
        losses_truncated: losses.iter().count() > MAX_DIAGNOSTICS,
    }
}

fn validate_relative_path(value: &str) -> Result<(), AppError> {
    let path = Path::new(value);
    if value.is_empty()
        || value.len() > 1_024
        || value.contains('\\')
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(client_error(
            "invalid_path",
            "Bundle paths must be normalized relative paths",
        ));
    }
    Ok(())
}

fn confined_join(root: &Path, relative: &str) -> Result<PathBuf, AppError> {
    validate_relative_path(relative)?;
    Ok(root.join(relative))
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn media_type(path: &str) -> String {
    mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string()
}

fn client_error(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::new(StatusCode::BAD_REQUEST, code, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reclaims_only_abandoned_private_scratch_roots() -> Result<(), Box<dyn std::error::Error>> {
        let parent = tempfile::tempdir()?;
        let current = parent.path().join(format!("{SCRATCH_PREFIX}current"));
        let abandoned = parent.path().join(format!("{SCRATCH_PREFIX}abandoned"));
        let reused_pid = parent
            .path()
            .join(format!("{SCRATCH_PREFIX}{}-old", std::process::id()));
        let unrelated = parent.path().join("unrelated");
        std::fs::create_dir_all(&current)?;
        std::fs::create_dir_all(&abandoned)?;
        std::fs::create_dir_all(&reused_pid)?;
        std::fs::create_dir_all(&unrelated)?;

        reclaim_scratch_roots(parent.path(), &current)?;

        assert!(current.is_dir());
        assert!(!abandoned.exists());
        assert!(!reused_pid.exists());
        assert!(unrelated.is_dir());
        Ok(())
    }
}
