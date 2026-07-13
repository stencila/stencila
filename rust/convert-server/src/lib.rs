#![recursion_limit = "256"]

use std::{
    collections::HashSet,
    fmt::{self, Display},
    io::{Cursor, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, FromRequest, Multipart, Query},
    http::{
        HeaderMap, HeaderValue, Request, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};
use bytes::Bytes;
use eyre::Report;
use serde::{Deserialize, Serialize};
use stencila_codec_arxiv::ArxivCodec;
use stencila_codec_openrxiv::OpenRxivCodec;
use stencila_codec_pmc::PmcCodec;
use stencila_codecs::{self, CodecDirection, DecodeOptions, EncodeOptions, Format};
use stencila_version::STENCILA_VERSION;
use strum::IntoEnumIterator;
use tempfile::{TempDir, tempdir};
use tokio::{fs, time::timeout};
use zip::{ZipWriter, write::SimpleFileOptions};

pub const MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = MAX_UPLOAD_BYTES + 1024 * 1024;
pub const CONVERSION_TIMEOUT: Duration = Duration::from_secs(60);
pub const FORMAT_DENYLIST: &[Format] = &[Format::Directory];

pub fn app() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/formats", get(formats))
        .route("/api/convert", post(convert))
        .fallback(fallback)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: STENCILA_VERSION,
    })
}

async fn formats(Query(query): Query<FormatsQuery>) -> Json<Vec<FormatInfo>> {
    Json(supported_formats_with_allowlists(
        parse_format_allowlist(query.from.as_deref()),
        parse_format_allowlist(query.to.as_deref()),
    ))
}

async fn fallback(request: Request<Body>) -> Response {
    let path = request.uri().path();

    if !path.starts_with("/api/")
        && let Some(response) = development_static_response(path).await
    {
        return response;
    }

    api_not_found().into_response()
}

fn api_not_found() -> AppError {
    AppError::new(
        StatusCode::NOT_FOUND,
        "not_found",
        "The requested API endpoint was not found",
    )
}

#[cfg(debug_assertions)]
async fn development_static_response(path: &str) -> Option<Response> {
    use std::path::Component;

    let relative_path = match path {
        "" | "/" => PathBuf::from("index.html"),
        path => {
            let path = path.trim_start_matches('/');
            let path = PathBuf::from(path);
            if path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            }) {
                return None;
            }
            path
        }
    };

    let public_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../workers/convert/public")
        .canonicalize()
        .ok()?;
    let path = public_dir.join(relative_path);

    if !path.starts_with(&public_dir) || !path.is_file() {
        return None;
    }

    let bytes = fs::read(&path).await.ok()?;
    let content_type = mime_guess::from_path(&path).first_or_octet_stream();
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(content_type.as_ref()).ok()?,
    );
    headers.insert("Cache-Control", HeaderValue::from_static("no-store"));

    Some((headers, bytes).into_response())
}

#[cfg(not(debug_assertions))]
async fn development_static_response(_path: &str) -> Option<Response> {
    None
}

async fn convert(request: Request<Body>) -> Result<Response, AppError> {
    let raw = raw_request_from_http(request).await?;
    let request = prepare_request(raw).await?;
    let result = timeout(CONVERSION_TIMEOUT, convert_request(request)).await;

    match result {
        Ok(result) => result,
        Err(_) => Err(conversion_timeout_error()),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    ok: bool,
    version: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatInfo {
    name: String,
    label: String,
    extension: String,
    media_type: String,
    from: bool,
    to: bool,
}

pub fn supported_formats() -> Vec<FormatInfo> {
    supported_formats_with_allowlists(None, None)
}

#[derive(Debug, Default, Deserialize)]
struct FormatsQuery {
    from: Option<String>,
    to: Option<String>,
}

fn parse_format_allowlist(value: Option<&str>) -> Option<HashSet<Format>> {
    value.map(|value| {
        value
            .split(',')
            .map(Format::from_name)
            .filter(|format| !(format.is_unknown() || format.is_other()))
            .collect()
    })
}

fn supported_formats_with_allowlists(
    from_allowlist: Option<HashSet<Format>>,
    to_allowlist: Option<HashSet<Format>>,
) -> Vec<FormatInfo> {
    Format::iter()
        .filter(|format| !(format.is_unknown() || format.is_other() || format_is_denied(format)))
        .filter_map(|format| {
            let from = from_allowlist
                .as_ref()
                .is_none_or(|allowlist| allowlist.contains(&format))
                && format_is_available(&format, CodecDirection::Decode);
            let to = to_allowlist
                .as_ref()
                .is_none_or(|allowlist| allowlist.contains(&format))
                && format_is_available(&format, CodecDirection::Encode);

            (from || to).then_some(FormatInfo {
                name: format.to_string(),
                label: format.name().to_string(),
                extension: format.extension(),
                media_type: format.media_type(),
                from,
                to,
            })
        })
        .collect()
}

fn format_is_denied(format: &Format) -> bool {
    FORMAT_DENYLIST.contains(format)
}

fn format_is_available(format: &Format, direction: CodecDirection) -> bool {
    stencila_codecs::get(None, Some(format), Some(direction))
        .is_ok_and(|codec| codec.is_available())
}

#[derive(Debug)]
struct RawConvertRequest {
    source: Option<RawInput>,
    to: Option<String>,
    from: Option<String>,
    mode: Option<String>,
    compact: Option<bool>,
    standalone: Option<bool>,
    embed_media: Option<bool>,
}

impl RawConvertRequest {
    fn empty() -> Self {
        Self {
            source: None,
            to: None,
            from: None,
            mode: None,
            compact: None,
            standalone: None,
            embed_media: None,
        }
    }

    fn set_source(&mut self, source: RawInput) -> Result<(), AppError> {
        if self.source.is_some() {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "multiple_inputs",
                "Provide exactly one input: either file or url",
            ));
        }
        self.source = Some(source);
        Ok(())
    }
}

#[derive(Debug)]
enum RawInput {
    Upload {
        filename: Option<String>,
        bytes: Bytes,
    },
    Url(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonConvertRequest {
    url: String,
    to: String,
    from: Option<String>,
    mode: Option<String>,
    compact: Option<bool>,
    standalone: Option<bool>,
    embed_media: Option<bool>,
}

impl From<JsonConvertRequest> for RawConvertRequest {
    fn from(request: JsonConvertRequest) -> Self {
        Self {
            source: Some(RawInput::Url(request.url)),
            to: Some(request.to),
            from: request.from,
            mode: request.mode,
            compact: request.compact,
            standalone: request.standalone,
            embed_media: request.embed_media,
        }
    }
}

async fn raw_request_from_http(request: Request<Body>) -> Result<RawConvertRequest, AppError> {
    let content_type = request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();

    if content_type.starts_with("multipart/form-data") {
        let multipart = Multipart::from_request(request, &())
            .await
            .map_err(|error| extractor_error(error.status(), error.body_text()))?;
        raw_request_from_multipart(multipart).await
    } else if content_type.starts_with("application/json") {
        let Json(request) = Json::<JsonConvertRequest>::from_request(request, &())
            .await
            .map_err(|error| extractor_error(error.status(), error.body_text()))?;
        Ok(request.into())
    } else {
        Err(AppError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Use multipart/form-data or application/json",
        ))
    }
}

async fn raw_request_from_multipart(
    mut multipart: Multipart,
) -> Result<RawConvertRequest, AppError> {
    let mut request = RawConvertRequest::empty();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| extractor_error(error.status(), error.body_text()))?
    {
        let Some(name) = field.name().map(str::to_string) else {
            continue;
        };

        match name.as_str() {
            "file" => {
                let filename = field.file_name().map(str::to_string);
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|error| extractor_error(error.status(), error.body_text()))?;
                check_upload_size(bytes.len())?;
                request.set_source(RawInput::Upload { filename, bytes })?;
            }
            "url" => {
                let url = field
                    .text()
                    .await
                    .map_err(|error| extractor_error(error.status(), error.body_text()))?;
                if !url.trim().is_empty() {
                    request.set_source(RawInput::Url(url))?;
                }
            }
            "to" => {
                request.to = Some(text_field(field).await?);
            }
            "from" => {
                let value = text_field(field).await?;
                if !value.trim().is_empty() {
                    request.from = Some(value);
                }
            }
            "mode" => {
                let value = text_field(field).await?;
                if !value.trim().is_empty() {
                    request.mode = Some(value);
                }
            }
            "compact" => {
                request.compact = parse_bool_field("compact", &text_field(field).await?)?;
            }
            "standalone" => {
                request.standalone = parse_bool_field("standalone", &text_field(field).await?)?;
            }
            "embedMedia" => {
                request.embed_media = parse_bool_field("embedMedia", &text_field(field).await?)?;
            }
            _ => {}
        }
    }

    Ok(request)
}

async fn text_field(field: axum::extract::multipart::Field<'_>) -> Result<String, AppError> {
    field
        .text()
        .await
        .map_err(|error| extractor_error(error.status(), error.body_text()))
}

fn extractor_error(status: StatusCode, message: String) -> AppError {
    if status == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "input_too_large",
            "Input exceeds the 25 MiB public service limit",
        )
    } else {
        AppError::new(status, "invalid_request", message)
    }
}

pub fn check_upload_size(size: usize) -> Result<(), AppError> {
    if size > MAX_UPLOAD_BYTES {
        Err(AppError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "input_too_large",
            "Input exceeds the 25 MiB public service limit",
        ))
    } else {
        Ok(())
    }
}

fn parse_bool_field(field: &'static str, value: &str) -> Result<Option<bool>, AppError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" => Ok(None),
        "1" | "true" | "yes" | "on" => Ok(Some(true)),
        "0" | "false" | "no" | "off" => Ok(Some(false)),
        _ => Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            format!("Field `{field}` must be a boolean"),
        )),
    }
}

struct ConvertRequest {
    source: InputSource,
    to: Format,
    from: Option<Format>,
    mode: Option<Mode>,
    compact: Option<bool>,
    standalone: Option<bool>,
    embed_media: Option<bool>,
}

enum InputSource {
    Upload {
        path: PathBuf,
        filename: String,
        _temp_dir: TempDir,
    },
    Identifier {
        identifier: String,
        filename: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentifierKind {
    Arxiv,
    OpenRxiv,
    Pmc,
}

impl IdentifierKind {
    fn from_identifier(identifier: &str) -> Option<Self> {
        if ArxivCodec::supports_identifier(identifier) {
            Some(Self::Arxiv)
        } else if OpenRxivCodec::supports_identifier(identifier) {
            Some(Self::OpenRxiv)
        } else if PmcCodec::supports_identifier(identifier) {
            Some(Self::Pmc)
        } else {
            None
        }
    }
}

async fn prepare_request(raw: RawConvertRequest) -> Result<ConvertRequest, AppError> {
    let to = raw
        .to
        .as_deref()
        .ok_or_else(|| {
            AppError::new(
                StatusCode::BAD_REQUEST,
                "missing_format",
                "The `to` output format is required",
            )
        })
        .and_then(|value| parse_format(value, CodecDirection::Encode))?;

    let from = raw
        .from
        .as_deref()
        .map(|value| parse_format(value, CodecDirection::Decode))
        .transpose()?;

    let mode = raw.mode.as_deref().map(Mode::parse).transpose()?;

    let source = match raw.source {
        Some(RawInput::Upload { filename, bytes }) => {
            check_upload_size(bytes.len())?;
            let filename = sanitize_filename(filename.as_deref().unwrap_or("upload"));
            let temp_dir = tempdir().map_err(AppError::internal)?;
            let path = upload_path(temp_dir.path(), &filename, from.as_ref());
            fs::write(&path, bytes).await.map_err(AppError::internal)?;
            InputSource::Upload {
                path,
                filename,
                _temp_dir: temp_dir,
            }
        }
        Some(RawInput::Url(identifier)) => {
            let identifier = identifier.trim().to_string();
            let kind = IdentifierKind::from_identifier(&identifier).ok_or_else(|| {
                AppError::new(
                    StatusCode::BAD_REQUEST,
                    "unsupported_url",
                    "Only arXiv, bioRxiv, medRxiv, and PMC identifiers or URLs are supported",
                )
            })?;
            InputSource::Identifier {
                filename: identifier_filename(kind, &identifier),
                identifier,
            }
        }
        None => {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "missing_input",
                "Provide exactly one input: either file or url",
            ));
        }
    };

    Ok(ConvertRequest {
        source,
        to,
        from,
        mode,
        compact: raw.compact,
        standalone: raw.standalone,
        embed_media: raw.embed_media,
    })
}

fn upload_path(temp_dir: &Path, filename: &str, from: Option<&Format>) -> PathBuf {
    if Path::new(filename).extension().is_some() {
        temp_dir.join(filename)
    } else if let Some(from) = from {
        temp_dir.join(format!(
            "{filename}.{extension}",
            extension = from.extension()
        ))
    } else {
        temp_dir.join(filename)
    }
}

fn parse_format(value: &str, direction: CodecDirection) -> Result<Format, AppError> {
    let format = Format::from_name(value);
    if format.is_unknown() || format.is_other() || format_is_denied(&format) {
        return Err(unsupported_format(value));
    }

    if !format_is_available(&format, direction) {
        return Err(unsupported_format(value));
    }

    Ok(format)
}

fn unsupported_format(value: &str) -> AppError {
    AppError::new(
        StatusCode::BAD_REQUEST,
        "unsupported_format",
        format!("Unsupported format `{}`", value.trim()),
    )
}

fn conversion_timeout_error() -> AppError {
    AppError::new(
        StatusCode::GATEWAY_TIMEOUT,
        "conversion_timeout",
        "Conversion exceeded the public service time limit",
    )
}

async fn convert_request(request: ConvertRequest) -> Result<Response, AppError> {
    let decode_options = Some(DecodeOptions {
        format: request.from.clone(),
        reproducible: Some(false),
        ignore_artifacts: Some(true),
        no_artifacts: Some(true),
        ..Default::default()
    });

    let (node, input_filename) = match request.source {
        InputSource::Upload { path, filename, .. } => (
            stencila_codecs::from_path(&path, decode_options)
                .await
                .map_err(AppError::conversion)?,
            filename,
        ),
        InputSource::Identifier {
            identifier,
            filename,
        } => (
            stencila_codecs::from_identifier(&identifier, decode_options)
                .await
                .map_err(AppError::conversion)?,
            filename,
        ),
    };

    let codec = stencila_codecs::get(None, Some(&request.to), Some(CodecDirection::Encode))
        .map_err(AppError::conversion)?;
    let can_inline = !request.to.is_binary() && codec.supports_to_string();
    let disposition = response_disposition(request.mode, &request.to, can_inline);
    let output_filename = output_filename(&input_filename, &request.to);
    let encode_options = Some(EncodeOptions {
        format: Some(request.to.clone()),
        compact: request.compact,
        standalone: request.standalone,
        embed_media: request.embed_media,
        reproducible: Some(false),
        ..Default::default()
    });

    let bytes = if disposition == Disposition::Inline {
        stencila_codecs::to_string(&node, encode_options)
            .await
            .map_err(AppError::conversion)?
            .into_bytes()
    } else {
        let temp_dir = tempdir().map_err(AppError::internal)?;
        let path = temp_dir.path().join(&output_filename);
        stencila_codecs::to_path(&node, &path, encode_options)
            .await
            .map_err(AppError::conversion)?;
        let files = files_in_directory(temp_dir.path()).await?;
        if files.len() > 1 {
            let archive_filename = archive_filename(&output_filename, &request.to);
            let bytes = zip_files(temp_dir.path(), files).await?;
            return response_from_bytes_with_media_type(
                bytes,
                "application/zip",
                Disposition::Attachment,
                &archive_filename,
            );
        }
        fs::read(path).await.map_err(AppError::internal)?
    };

    response_from_bytes(bytes, &request.to, disposition, &output_filename)
}

fn response_from_bytes(
    bytes: Vec<u8>,
    format: &Format,
    disposition: Disposition,
    output_filename: &str,
) -> Result<Response, AppError> {
    response_from_bytes_with_media_type(bytes, &format.media_type(), disposition, output_filename)
}

fn response_from_bytes_with_media_type(
    bytes: Vec<u8>,
    media_type: &str,
    disposition: Disposition,
    output_filename: &str,
) -> Result<Response, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(media_type).map_err(AppError::internal)?,
    );
    headers.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition(disposition, output_filename))
            .map_err(AppError::internal)?,
    );
    headers.insert(
        "X-Stencila-Output-Filename",
        HeaderValue::from_str(output_filename).map_err(AppError::internal)?,
    );

    Ok((headers, bytes).into_response())
}

async fn files_in_directory(root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut directories = vec![root.to_path_buf()];
    let mut files = Vec::new();

    while let Some(directory) = directories.pop() {
        let mut entries = fs::read_dir(directory).await.map_err(AppError::internal)?;
        while let Some(entry) = entries.next_entry().await.map_err(AppError::internal)? {
            let file_type = entry.file_type().await.map_err(AppError::internal)?;
            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                files.push(entry.path());
            }
        }
    }

    files.sort();
    Ok(files)
}

async fn zip_files(root: &Path, paths: Vec<PathBuf>) -> Result<Vec<u8>, AppError> {
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let relative = path
            .strip_prefix(root)
            .map_err(AppError::internal)?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(path).await.map_err(AppError::internal)?;
        files.push((relative, bytes));
    }

    tokio::task::spawn_blocking(move || -> Result<Vec<u8>, AppError> {
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();
        for (path, bytes) in files {
            archive
                .start_file(path, options)
                .map_err(AppError::internal)?;
            archive.write_all(&bytes).map_err(AppError::internal)?;
        }
        archive
            .finish()
            .map(Cursor::into_inner)
            .map_err(AppError::internal)
    })
    .await
    .map_err(AppError::internal)?
}

fn archive_filename(output_filename: &str, format: &Format) -> String {
    let suffix = format!(".{}", format.extension());
    let stem = output_filename
        .strip_suffix(&suffix)
        .unwrap_or(output_filename);
    format!("{stem}.zip")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Inline,
    Download,
}

impl Mode {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "inline" => Ok(Self::Inline),
            "download" => Ok(Self::Download),
            _ => Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "Field `mode` must be `inline` or `download`",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    Inline,
    Attachment,
}

pub fn response_disposition(mode: Option<Mode>, format: &Format, can_inline: bool) -> Disposition {
    match mode {
        Some(Mode::Download) => Disposition::Attachment,
        Some(Mode::Inline) | None if can_inline && !format.is_binary() => Disposition::Inline,
        Some(Mode::Inline) | None => Disposition::Attachment,
    }
}

pub fn content_disposition(disposition: Disposition, filename: &str) -> String {
    let disposition = match disposition {
        Disposition::Inline => "inline",
        Disposition::Attachment => "attachment",
    };
    format!("{disposition}; filename=\"{}\"", filename.replace('"', ""))
}

pub fn sanitize_filename(filename: &str) -> String {
    let name = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("upload");

    let mut sanitized = String::with_capacity(name.len());
    let mut previous_dash = false;

    for char in name.chars() {
        let replacement = if char.is_ascii_alphanumeric() || matches!(char, '.' | '_' | '-') {
            char
        } else {
            '-'
        };

        if replacement == '-' {
            if previous_dash {
                continue;
            }
            previous_dash = true;
        } else {
            previous_dash = false;
        }

        sanitized.push(replacement);
    }

    let sanitized = sanitized.trim_matches(['.', '-']).to_string();
    let sanitized = if sanitized.is_empty() {
        "upload".to_string()
    } else {
        sanitized
    };

    truncate_filename_preserving_extension(&sanitized, 120)
}

fn truncate_filename_preserving_extension(filename: &str, max_len: usize) -> String {
    if filename.len() <= max_len {
        return filename.to_string();
    }

    let detected_format = Format::from_path(Path::new(filename));
    let recognized_suffix = filename.match_indices('.').find_map(|(index, _)| {
        let suffix = &filename[index + 1..];
        (Format::from_name(suffix) == detected_format
            && !(detected_format.is_unknown() || detected_format.is_other()))
        .then_some(&filename[index..])
    });
    let suffix = recognized_suffix.or_else(|| {
        Path::new(filename)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| &filename[filename.len() - extension.len() - 1..])
    });

    if let Some(suffix) = suffix
        && suffix.len() < max_len
    {
        let stem_len = max_len - suffix.len();
        return format!("{}{suffix}", &filename[..stem_len]);
    }

    filename[..max_len].to_string()
}

pub fn output_filename(input_filename: &str, format: &Format) -> String {
    let sanitized = sanitize_filename(input_filename);
    let stem = Path::new(&sanitized)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("converted");

    format!("{stem}.{extension}", extension = format.extension())
}

fn arxiv_filename(identifier: &str) -> String {
    let sanitized = sanitize_filename(identifier);
    let stem = sanitized
        .strip_prefix("arXiv-")
        .or_else(|| sanitized.strip_prefix("arxiv-"))
        .unwrap_or(&sanitized);
    format!("arxiv-{stem}")
}

fn identifier_filename(kind: IdentifierKind, identifier: &str) -> String {
    match kind {
        IdentifierKind::Arxiv => arxiv_filename(identifier),
        IdentifierKind::OpenRxiv => format!("openrxiv-{}", sanitize_filename(identifier)),
        IdentifierKind::Pmc => format!("pmc-{}", sanitize_filename(identifier)),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    error: ErrorDetails,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorDetails {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl AppError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    fn internal(error: impl Display) -> Self {
        tracing::error!("Internal convert server error: {error}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Internal server error",
        )
    }

    fn conversion(error: Report) -> Self {
        tracing::warn!("Conversion failed: {error}");
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "conversion_failed",
            error.to_string(),
        )
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = Json(ErrorBody {
            error: ErrorDetails {
                code: self.code,
                message: self.message,
            },
        });
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use stencila_codecs::Format;
    use tokio::time::{Duration, sleep, timeout};

    use super::*;

    #[test]
    fn validates_output_format() -> Result<(), AppError> {
        assert_eq!(parse_format("json", CodecDirection::Encode)?, Format::Json);
        let error = parse_format("not-a-format", CodecDirection::Encode)
            .err()
            .ok_or_else(|| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "test", "test"))?;

        assert_eq!(error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.code(), "unsupported_format");
        Ok(())
    }

    #[test]
    fn sanitizes_filenames() {
        assert_eq!(sanitize_filename("../unsafe name!.md"), "unsafe-name-.md");
        assert_eq!(sanitize_filename(""), "upload");
        assert_eq!(output_filename("paper.md", &Format::Json), "paper.json");
        assert_eq!(output_filename("paper", &Format::Jats), "paper.jats.xml");

        let long_markdown = format!("{}.md", "a".repeat(150));
        let sanitized = sanitize_filename(&long_markdown);
        assert_eq!(sanitized.len(), 120);
        assert!(sanitized.ends_with(".md"));

        let long_oxa = format!("{}.oxa.json", "a".repeat(150));
        let sanitized = sanitize_filename(&long_oxa);
        assert_eq!(sanitized.len(), 120);
        assert!(sanitized.ends_with(".oxa.json"));
    }

    #[test]
    fn names_archives_after_the_output_stem() {
        assert_eq!(
            archive_filename("my.paper.md", &Format::Markdown),
            "my.paper.zip"
        );
        assert_eq!(
            archive_filename("my.paper.jats.xml", &Format::Jats),
            "my.paper.zip"
        );
    }

    #[test]
    fn denies_formats_unsupported_by_the_public_api() {
        assert!(format_is_denied(&Format::Directory));
        assert!(
            supported_formats()
                .iter()
                .all(|format| format.name != "directory")
        );

        let error = parse_format("directory", CodecDirection::Decode)
            .err()
            .unwrap_or_else(|| unreachable!("directory should be denied"));
        assert_eq!(error.code(), "unsupported_format");
    }

    #[test]
    fn allows_supported_remote_identifiers() {
        assert_eq!(
            IdentifierKind::from_identifier("2507.11254"),
            Some(IdentifierKind::Arxiv)
        );
        assert_eq!(
            IdentifierKind::from_identifier(
                "https://www.biorxiv.org/content/10.1101/2025.07.15.664907v1"
            ),
            Some(IdentifierKind::OpenRxiv)
        );
        assert_eq!(
            IdentifierKind::from_identifier(
                "https://www.biorxiv.org/content/10.64898/2026.07.07.736512v1"
            ),
            Some(IdentifierKind::OpenRxiv)
        );
        assert_eq!(
            IdentifierKind::from_identifier(
                "https://www.medrxiv.org/content/10.1101/2024.12.01.24318123v2"
            ),
            Some(IdentifierKind::OpenRxiv)
        );
        assert_eq!(
            IdentifierKind::from_identifier("10.1101/2025.07.15.664907"),
            Some(IdentifierKind::OpenRxiv)
        );
        assert_eq!(
            IdentifierKind::from_identifier("PMC1234567"),
            Some(IdentifierKind::Pmc)
        );
        assert_eq!(
            IdentifierKind::from_identifier("https://pmc.ncbi.nlm.nih.gov/articles/PMC1234567/"),
            Some(IdentifierKind::Pmc)
        );
        assert_eq!(
            IdentifierKind::from_identifier("https://example.com/paper.md"),
            None
        );
    }

    #[test]
    fn maps_upload_limit_status_and_code() -> Result<(), AppError> {
        let error = check_upload_size(MAX_UPLOAD_BYTES + 1)
            .err()
            .ok_or_else(|| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "test", "test"))?;

        assert_eq!(error.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(error.code(), "input_too_large");
        Ok(())
    }

    #[tokio::test]
    async fn maps_timeout() -> Result<(), AppError> {
        let result = timeout(Duration::from_millis(1), sleep(Duration::from_millis(20))).await;
        assert!(result.is_err());
        let error = conversion_timeout_error();
        assert_eq!(error.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(error.code(), "conversion_timeout");
        Ok(())
    }

    #[test]
    fn chooses_response_disposition() {
        assert_eq!(
            response_disposition(None, &Format::Markdown, true),
            Disposition::Inline
        );
        assert_eq!(
            response_disposition(Some(Mode::Download), &Format::Markdown, true),
            Disposition::Attachment
        );
        assert_eq!(
            response_disposition(Some(Mode::Inline), &Format::Pdf, false),
            Disposition::Attachment
        );
        assert_eq!(
            content_disposition(Disposition::Inline, "paper.md"),
            "inline; filename=\"paper.md\""
        );
    }
}
