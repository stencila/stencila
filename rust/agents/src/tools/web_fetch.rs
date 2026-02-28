//! `web_fetch` tool: fetch URL content and save to `.stencila/cache/web/`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

use http_cache_semantics::{AfterResponse, BeforeRequest, CachePolicy};
use reqwest::Client;
use reqwest::header::HeaderMap;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use stencila_codec_html as codec_html;
use stencila_codec_markdown as codec_markdown;
use stencila_models3::types::tool::ToolDefinition;
use stencila_schema::{Block, ImageObject, Inline, Node, VisitorMut, WalkControl};
use tokio::fs::{create_dir_all, write};
use tracing::warn;
use url::Url;

use crate::error::{AgentError, AgentResult};
use crate::registry::{ToolExecutorFn, ToolOutput};

use super::required_str;

/// Maximum response body size: 10 MB.
const MAX_RESPONSE_BYTES: u64 = 10 * 1024 * 1024;

/// HTTP request timeout: 30 seconds.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Shared HTTP client — reuses connection pool across all requests.
static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(stencila_version::STENCILA_USER_AGENT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .expect("failed to build HTTP client")
});

/// Tool definition matching `tests/fixtures/tool_schemas/web_fetch.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "web_fetch".into(),
        description: "Fetch a URL and save the content to .stencila/cache/web/. \
            HTML is converted to Markdown with images extracted to a media directory. \
            Other content types are saved as-is. Returns a manifest of saved files. \
            Use read_file, grep, or glob to explore the content."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch."
                },
                "raw": {
                    "type": "boolean",
                    "description": "If true, save the response body as-is without conversion. If false (default), convert HTML to Markdown and extract media assets.",
                    "default": false
                }
            },
            "required": ["url"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let url_str = required_str(&args, "url")?;
                let raw = args.get("raw").and_then(Value::as_bool).unwrap_or(false);

                // Validate URL
                let url = Url::parse(url_str).map_err(|e| AgentError::ValidationError {
                    reason: format!("invalid URL: {e}"),
                })?;
                if url.scheme() != "http" && url.scheme() != "https" {
                    return Err(AgentError::ValidationError {
                        reason: format!("unsupported URL scheme: {}", url.scheme()),
                    });
                }

                // Resolve cache directory
                let working_dir = Path::new(env.working_directory());
                let stencila_dir = stencila_dirs::closest_stencila_dir(working_dir, true)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!("failed to find .stencila dir: {e}"),
                    })?;
                let web_cache = stencila_dir.join("cache").join("web");
                let key = cache_key(url_str, raw);
                let cache_dir = web_cache.join(&key);
                create_dir_all(&cache_dir)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!("failed to create cache dir: {e}"),
                    })?;

                let policy_path = cache_dir.join("_cache_policy.json");

                // Build an http::Request for cache semantics
                let http_req = build_http_request(url_str)?;

                // Check cache
                if let Some(policy) = load_policy(&policy_path).await {
                    match policy.before_request(&http_req, SystemTime::now()) {
                        BeforeRequest::Fresh(_parts) => {
                            if content_files_exist(&cache_dir).await {
                                let ttl = policy.time_to_live(SystemTime::now());
                                let status =
                                    format!("cached, fresh — expires in {}", format_duration(ttl));
                                return Ok(ToolOutput::Text(
                                    build_manifest(&cache_dir, working_dir, url_str, &status).await,
                                ));
                            }
                            // Files missing — fall through to fresh fetch
                        }
                        BeforeRequest::Stale { request: parts, .. } => {
                            // Build revalidation request with headers from cache policy
                            let response =
                                send_request_with_headers(url_str, parts.headers).await?;
                            let http_resp = to_http_response(&response);
                            match policy.after_response(&http_req, &http_resp, SystemTime::now()) {
                                AfterResponse::NotModified(new_policy, _) => {
                                    if content_files_exist(&cache_dir).await {
                                        save_policy(&policy_path, &new_policy).await;
                                        return Ok(ToolOutput::Text(
                                            build_manifest(
                                                &cache_dir,
                                                working_dir,
                                                url_str,
                                                "304 Not Modified, revalidated",
                                            )
                                            .await,
                                        ));
                                    }
                                    // Files missing — fall through to fresh fetch
                                }
                                AfterResponse::Modified(new_policy, _) => {
                                    let status_code = response.status();
                                    // Non-2xx during revalidation: return error,
                                    // preserve existing cache.
                                    check_http_status(status_code, url_str)?;
                                    let final_url = response.url().to_string();
                                    let body = read_body(response, url_str).await?;
                                    let content_type = http_resp
                                        .headers()
                                        .get("content-type")
                                        .and_then(|v| v.to_str().ok())
                                        .unwrap_or("")
                                        .to_string();
                                    process_response(
                                        &body,
                                        &content_type,
                                        &final_url,
                                        &cache_dir,
                                        raw,
                                    )
                                    .await?;
                                    if new_policy.is_storable() {
                                        save_policy(&policy_path, &new_policy).await;
                                    } else {
                                        remove_policy(&policy_path).await;
                                    }
                                    let status = format!(
                                        "{} {}, {}",
                                        status_code.as_u16(),
                                        status_code.canonical_reason().unwrap_or("OK"),
                                        if new_policy.is_storable() {
                                            format!(
                                                "cached for {}",
                                                format_duration(
                                                    new_policy.time_to_live(SystemTime::now())
                                                )
                                            )
                                        } else {
                                            "not cacheable".into()
                                        }
                                    );
                                    return Ok(ToolOutput::Text(
                                        build_manifest(&cache_dir, working_dir, url_str, &status)
                                            .await,
                                    ));
                                }
                            }
                        }
                    }
                }

                // Fresh fetch (also reached on cache-miss fall-through)
                let response = send_request(url_str).await?;
                let status_code = response.status();

                // Map HTTP errors
                check_http_status(status_code, url_str)?;

                let http_resp = to_http_response(&response);
                let content_type = response
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();
                // Use the final URL after redirects for resolving relative image URLs
                let final_url = response.url().to_string();

                let body = read_body(response, url_str).await?;

                let policy = CachePolicy::new(&http_req, &http_resp);
                process_response(&body, &content_type, &final_url, &cache_dir, raw).await?;

                if policy.is_storable() {
                    save_policy(&policy_path, &policy).await;
                } else {
                    remove_policy(&policy_path).await;
                }

                let status = format!(
                    "{} {}, {}",
                    status_code.as_u16(),
                    status_code.canonical_reason().unwrap_or("OK"),
                    if policy.is_storable() {
                        format!(
                            "cached for {}",
                            format_duration(policy.time_to_live(SystemTime::now()))
                        )
                    } else {
                        "not cacheable".into()
                    }
                );

                Ok(ToolOutput::Text(
                    build_manifest(&cache_dir, working_dir, url_str, &status).await,
                ))
            })
        },
    )
}

// ---------------------------------------------------------------------------
// Cache key
// ---------------------------------------------------------------------------

fn cache_key(url: &str, raw: bool) -> String {
    // Hash of canonical key
    let mut hasher = Sha256::new();
    hasher.update(format!("{url}\0{raw}"));
    let hash = format!("{:x}", hasher.finalize());
    let hash_short = &hash[..12];

    // Human-readable prefix from URL
    let prefix = human_readable_prefix(url);

    format!("{prefix}_{hash_short}")
}

fn human_readable_prefix(url: &str) -> String {
    let parsed = Url::parse(url).ok();
    let host = parsed
        .as_ref()
        .and_then(|u| u.host_str())
        .unwrap_or("unknown");
    let path = parsed.as_ref().map(|u| u.path()).unwrap_or("");

    // Build prefix from host + path segments
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    let mut prefix = host.to_string();
    for seg in &segments {
        let candidate = format!("{prefix}_{seg}");
        if candidate.len() > 80 {
            break;
        }
        prefix = candidate;
    }

    // Sanitize: keep only alphanumeric, hyphens, dots, underscores
    prefix
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

/// Map an HTTP status code to an `AgentError` for non-2xx responses.
fn check_http_status(status: reqwest::StatusCode, url: &str) -> AgentResult<()> {
    match status.as_u16() {
        200..=299 => Ok(()),
        404 => Err(AgentError::FileNotFound { path: url.into() }),
        401 | 403 => Err(AgentError::PermissionDenied { path: url.into() }),
        code => Err(AgentError::Io {
            message: format!(
                "HTTP {code}: {}",
                status.canonical_reason().unwrap_or("error")
            ),
        }),
    }
}

/// Build an `http::Request<()>` for use with `CachePolicy`.
fn build_http_request(url: &str) -> AgentResult<http::Request<()>> {
    http::Request::builder()
        .method(http::Method::GET)
        .uri(url)
        .header("user-agent", stencila_version::STENCILA_USER_AGENT)
        .body(())
        .map_err(|e| AgentError::Io {
            message: format!("failed to build request: {e}"),
        })
}

/// Convert a `reqwest::Response` (headers + status) to `http::Response<()>` for cache semantics.
///
/// Must be called *before* consuming the response body (e.g. via `read_body`),
/// since it borrows the response to copy headers and status.
fn to_http_response(resp: &reqwest::Response) -> http::Response<()> {
    let mut builder = http::Response::builder().status(resp.status());
    if let Some(headers) = builder.headers_mut() {
        headers.extend(resp.headers().iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    builder.body(()).expect("infallible response build")
}

async fn send_request(url: &str) -> AgentResult<reqwest::Response> {
    CLIENT.get(url).send().await.map_err(|e| AgentError::Io {
        message: format!("HTTP request failed: {e}"),
    })
}

async fn send_request_with_headers(
    url: &str,
    headers: HeaderMap,
) -> AgentResult<reqwest::Response> {
    CLIENT
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| AgentError::Io {
            message: format!("HTTP request failed: {e}"),
        })
}

async fn read_body(response: reqwest::Response, url: &str) -> AgentResult<Vec<u8>> {
    read_body_limited(response, url, MAX_RESPONSE_BYTES).await
}

/// Stream the response body in chunks, aborting once `limit` bytes are exceeded.
///
/// If a `Content-Length` header is present it is checked up-front so we can
/// fail fast without reading any body data. For chunked / HTTP/2 responses
/// where the length is unknown, the body is streamed incrementally so memory
/// usage stays bounded.
async fn read_body_limited(
    response: reqwest::Response,
    url: &str,
    limit: u64,
) -> AgentResult<Vec<u8>> {
    // Fast-reject when Content-Length is present and exceeds the limit.
    if let Some(len) = response.content_length()
        && len > limit
    {
        return Err(AgentError::Io {
            message: format!(
                "response too large ({}, max {}). Use `shell` with `curl -o` for large files.",
                format_bytes(len),
                format_bytes(limit)
            ),
        });
    }

    // Stream chunks, accumulating into a Vec with a hard byte cap.
    let mut buf: Vec<u8> = Vec::new();
    let mut stream = response;
    loop {
        let chunk = stream.chunk().await.map_err(|e| AgentError::Io {
            message: format!("failed to read response body from {url}: {e}"),
        })?;
        match chunk {
            Some(bytes) => {
                if buf.len() as u64 + bytes.len() as u64 > limit {
                    return Err(AgentError::Io {
                        message: format!(
                            "response too large (>{}, max {}). Use `shell` with `curl -o` for large files.",
                            format_bytes(limit),
                            format_bytes(limit)
                        ),
                    });
                }
                buf.extend_from_slice(&bytes);
            }
            None => break,
        }
    }

    Ok(buf)
}

// ---------------------------------------------------------------------------
// Cache policy persistence
// ---------------------------------------------------------------------------

async fn load_policy(path: &Path) -> Option<CachePolicy> {
    let data = tokio::fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&data).ok()
}

async fn save_policy(path: &Path, policy: &CachePolicy) {
    if let Ok(json) = serde_json::to_string(policy) {
        let _ = write(path, json).await;
    }
}

async fn remove_policy(path: &Path) {
    let _ = tokio::fs::remove_file(path).await;
}

/// Remove all non-underscore content files/directories from `cache_dir`.
///
/// Called before writing new fetch output so that stale content from a
/// prior fetch (e.g. old `index.md` + `index.media/`) does not persist
/// when the content type or body changes.
async fn clear_content_files(cache_dir: &Path) {
    let mut entries = match tokio::fs::read_dir(cache_dir).await {
        Ok(entries) => entries,
        Err(_) => return,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('_') || name == "." || name == ".." {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            let _ = tokio::fs::remove_dir_all(&path).await;
        } else {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
}

async fn content_files_exist(cache_dir: &Path) -> bool {
    let mut entries = match tokio::fs::read_dir(cache_dir).await {
        Ok(entries) => entries,
        Err(_) => return false,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with('_') && name != "." && name != ".." {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Content processing
// ---------------------------------------------------------------------------

async fn process_response(
    body: &[u8],
    content_type: &str,
    url: &str,
    cache_dir: &Path,
    raw: bool,
) -> AgentResult<()> {
    // Remove stale content from prior fetches before writing new output.
    clear_content_files(cache_dir).await;

    let mime = content_type.split(';').next().unwrap_or("").trim();

    if mime == "text/html" && !raw {
        // HTML → Markdown
        let html = String::from_utf8_lossy(body);
        html_to_markdown(&html, url, cache_dir).await
    } else {
        // Save as-is
        let filename = output_filename(mime, url, raw);
        let path = cache_dir.join(&filename);
        write(&path, body).await.map_err(|e| AgentError::Io {
            message: format!("failed to write {}: {e}", path.display()),
        })
    }
}

fn output_filename(mime: &str, url: &str, raw: bool) -> String {
    debug_assert!(
        mime != "text/html" || raw,
        "non-raw text/html should go through html_to_markdown, not output_filename"
    );
    if mime == "text/html" && raw {
        return "index.html".into();
    }

    match mime {
        "application/json" => "index.json".into(),
        "text/plain" => "index.txt".into(),
        "text/csv" => "index.csv".into(),
        "text/xml" | "application/xml" => "index.xml".into(),
        "application/pdf" => "index.pdf".into(),
        m if m.starts_with("image/") => {
            let ext = mime_to_extension(m);
            // Try to get a filename from URL
            let parsed = Url::parse(url).ok();
            let url_filename = parsed
                .as_ref()
                .and_then(|u| u.path_segments())
                .and_then(|mut s| s.next_back())
                .filter(|s| !s.is_empty() && s.contains('.'))
                .map(sanitize_filename);
            url_filename.unwrap_or_else(|| format!("image.{ext}"))
        }
        _ => {
            // Try to get filename from URL path
            let parsed = Url::parse(url).ok();
            let url_filename = parsed
                .as_ref()
                .and_then(|u| u.path_segments())
                .and_then(|mut s| s.next_back())
                .filter(|s| !s.is_empty() && s.contains('.'))
                .map(sanitize_filename);
            url_filename.unwrap_or_else(|| "index.bin".into())
        }
    }
}

/// Strip leading underscores from a URL-derived filename so it cannot
/// collide with internal metadata files (e.g. `_cache_policy.json`).
fn sanitize_filename(name: &str) -> String {
    let stripped = name.trim_start_matches('_');
    if stripped.is_empty() {
        "file".into()
    } else {
        stripped.into()
    }
}

fn mime_to_extension(mime: &str) -> &str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/bmp" => "bmp",
        "image/tiff" => "tiff",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        _ => "bin",
    }
}

// ---------------------------------------------------------------------------
// HTML → Markdown with image download
// ---------------------------------------------------------------------------

async fn html_to_markdown(html: &str, url: &str, cache_dir: &Path) -> AgentResult<()> {
    let output_path = cache_dir.join("index.md");
    let media_dir = cache_dir.join("index.media");

    // 1. HTML → Stencila Node
    let (mut node, _) = codec_html::decode(html, None).map_err(|e| AgentError::Io {
        message: format!("HTML decode failed: {e}"),
    })?;

    // 2. Collect image URLs (resolves relative URLs against page URL)
    let mut collector = ImageUrlCollector::new(url);
    collector.walk(&mut node);

    // 3. Download remote images and build original_url→local_path mapping
    if !collector.urls.is_empty() {
        create_dir_all(&media_dir)
            .await
            .map_err(|e| AgentError::Io {
                message: format!("failed to create media dir: {e}"),
            })?;

        let mut url_map: HashMap<String, String> = HashMap::new();
        for (original_src, absolute_url) in &collector.urls {
            if let Some(local_path) = download_image(absolute_url, &media_dir).await {
                let relative = format!(
                    "index.media/{}",
                    local_path.file_name().unwrap_or_default().to_string_lossy()
                );
                url_map.insert(original_src.clone(), relative);
            } else {
                // Download failed — rewrite to the absolute remote URL so
                // the Markdown link remains valid rather than pointing at a
                // broken relative path.
                url_map.insert(original_src.clone(), absolute_url.clone());
            }
        }

        // 4. Rewrite URLs in the node tree
        if !url_map.is_empty() {
            let mut rewriter = ImageUrlRewriter { url_map: &url_map };
            rewriter.walk(&mut node);
        }
    }

    // 5. Extract any data: URI images
    let _ = stencila_node_media::extract_media(&mut node, Some(&output_path), &media_dir);

    // 6. Node → Markdown
    let (md, _) = codec_markdown::encode(&node, None).map_err(|e| AgentError::Io {
        message: format!("Markdown encode failed: {e}"),
    })?;

    write(&output_path, md).await.map_err(|e| AgentError::Io {
        message: format!("failed to write index.md: {e}"),
    })?;

    Ok(())
}

/// Collect all remote image URLs from a Node tree, resolving relative URLs
/// against the page base URL.
struct ImageUrlCollector {
    base_url: Option<Url>,
    /// Collected entries: (original_content_url, resolved_absolute_url).
    urls: Vec<(String, String)>,
}

impl ImageUrlCollector {
    fn new(page_url: &str) -> Self {
        Self {
            base_url: Url::parse(page_url).ok(),
            urls: Vec::new(),
        }
    }

    fn check_image(&mut self, img: &ImageObject) {
        let src = &img.content_url;

        // Skip data: URIs — handled by extract_media later
        if src.starts_with("data:") {
            return;
        }

        // Resolve to absolute URL
        let absolute = if src.starts_with("http://") || src.starts_with("https://") {
            src.clone()
        } else if let Some(base) = &self.base_url {
            match base.join(src) {
                Ok(resolved) => resolved.to_string(),
                Err(_) => return,
            }
        } else {
            return;
        };

        if !self.urls.iter().any(|(orig, _)| orig == src) {
            self.urls.push((src.clone(), absolute));
        }
    }
}

impl VisitorMut for ImageUrlCollector {
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::ImageObject(img) = inline {
            self.check_image(img);
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::ImageObject(img) = block {
            self.check_image(img);
        }
        WalkControl::Continue
    }

    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        if let Node::ImageObject(img) = node {
            self.check_image(img);
        }
        WalkControl::Continue
    }
}

/// Rewrite remote image URLs to local paths (or absolute remote URLs on
/// download failure).
struct ImageUrlRewriter<'a> {
    url_map: &'a HashMap<String, String>,
}

impl ImageUrlRewriter<'_> {
    fn rewrite_image(&self, img: &mut ImageObject) {
        if let Some(replacement) = self.url_map.get(&img.content_url) {
            img.content_url = replacement.clone();
        }
    }
}

impl VisitorMut for ImageUrlRewriter<'_> {
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::ImageObject(img) = inline {
            self.rewrite_image(img);
        }
        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::ImageObject(img) = block {
            self.rewrite_image(img);
        }
        WalkControl::Continue
    }

    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        if let Node::ImageObject(img) = node {
            self.rewrite_image(img);
        }
        WalkControl::Continue
    }
}

/// Download a single image, returning the local file path on success.
async fn download_image(image_url: &str, media_dir: &Path) -> Option<PathBuf> {
    let response = match CLIENT.get(image_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            warn!("failed to download image {image_url}: {e}");
            return None;
        }
    };
    if !response.status().is_success() {
        warn!(
            "failed to download image {image_url}: HTTP {}",
            response.status().as_u16()
        );
        return None;
    }

    // Determine filename from URL or content-type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    let bytes = match read_body_limited(response, image_url, MAX_RESPONSE_BYTES).await {
        Ok(b) => b,
        Err(e) => {
            warn!("skipping image {image_url}: {e}");
            return None;
        }
    };

    // Generate filename from URL hash + extension
    let ext = mime_to_extension(&content_type);
    let url_ext = Url::parse(image_url)
        .ok()
        .and_then(|u| {
            u.path_segments()?
                .next_back()
                .and_then(|s| s.rsplit_once('.'))
                .map(|(_, ext)| String::from(ext))
        })
        .filter(|e| !e.is_empty() && e.len() <= 5);

    let final_ext = url_ext.as_deref().unwrap_or(ext);
    let mut hasher = Sha256::new();
    hasher.update(image_url.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let filename = format!("{}_{}.{}", &hash[..8], &hash[8..16], final_ext);

    let path = media_dir.join(&filename);
    if let Err(e) = write(&path, &bytes).await {
        warn!("failed to write image {}: {e}", path.display());
        return None;
    }

    Some(path)
}

// ---------------------------------------------------------------------------
// Manifest building
// ---------------------------------------------------------------------------

async fn build_manifest(cache_dir: &Path, working_dir: &Path, url: &str, status: &str) -> String {
    let mut lines = vec![format!("Fetched {url} ({status})")];

    // Display paths relative to the working directory so they are portable
    // and directly usable in read_file / grep / glob calls.
    let rel_cache = relative_path(cache_dir, working_dir);

    // List content files
    if let Ok(mut entries) = tokio::fs::read_dir(cache_dir).await {
        let mut file_entries: Vec<(String, PathBuf)> = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('_') {
                continue;
            }
            file_entries.push((name, entry.path()));
        }
        file_entries.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, path) in &file_entries {
            let rel = relative_path(path, working_dir);
            if path.is_dir() {
                // List files in subdirectory (media dir)
                if let Ok(mut sub_entries) = tokio::fs::read_dir(path).await {
                    let mut sub_files: Vec<(String, PathBuf)> = Vec::new();
                    while let Ok(Some(sub_entry)) = sub_entries.next_entry().await {
                        let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                        sub_files.push((sub_name, sub_entry.path()));
                    }
                    sub_files.sort_by(|a, b| a.0.cmp(&b.0));
                    for (_, sub_path) in &sub_files {
                        let sub_rel = relative_path(sub_path, working_dir);
                        let size = tokio::fs::metadata(sub_path)
                            .await
                            .map(|m| m.len())
                            .unwrap_or(0);
                        lines.push(format!("  {} ({})", sub_rel.display(), format_bytes(size)));
                    }
                }
            } else {
                let metadata = tokio::fs::metadata(path).await.ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                if name.ends_with(".md")
                    || name.ends_with(".txt")
                    || name.ends_with(".json")
                    || name.ends_with(".csv")
                    || name.ends_with(".xml")
                    || name.ends_with(".html")
                {
                    // Count lines for text files
                    let content = tokio::fs::read_to_string(path).await.unwrap_or_default();
                    let line_count = content.lines().count();
                    let char_count = content.len();
                    lines.push(format!(
                        "  {} ({} chars, {} lines)",
                        rel.display(),
                        char_count,
                        line_count
                    ));
                } else if name.ends_with(".pdf") {
                    lines.push(format!(
                        "  {} ({}, use `shell` with `pdftotext` for text extraction)",
                        rel.display(),
                        format_bytes(size)
                    ));
                } else {
                    lines.push(format!("  {} ({})", rel.display(), format_bytes(size)));
                }
            }
        }
    }

    lines.push(String::new());
    lines.push(format!(
        "Use read_file, grep, or glob on {} to explore the content.",
        rel_cache.display()
    ));
    lines.join("\n")
}

/// Compute a path relative to a base directory. Falls back to the original
/// path if `strip_prefix` fails (e.g. different mount points).
fn relative_path(path: &Path, base: &Path) -> PathBuf {
    path.strip_prefix(base)
        .map(PathBuf::from)
        .unwrap_or_else(|_| path.to_path_buf())
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_deterministic() {
        let key1 = cache_key("https://example.com/page", false);
        let key2 = cache_key("https://example.com/page", false);
        assert_eq!(key1, key2);
    }

    #[test]
    fn cache_key_differs_by_raw() {
        let key_raw = cache_key("https://example.com/page", true);
        let key_normal = cache_key("https://example.com/page", false);
        assert_ne!(key_raw, key_normal);
    }

    #[test]
    fn cache_key_differs_by_query() {
        let key1 = cache_key("https://example.com/search?q=rust", false);
        let key2 = cache_key("https://example.com/search?q=python", false);
        assert_ne!(key1, key2);
    }

    #[test]
    fn cache_key_has_readable_prefix() {
        let key = cache_key("https://docs.rs/tokio/latest/tokio/", false);
        assert!(key.starts_with("docs.rs_tokio_latest_tokio_"));
        // 12-char hex hash at the end
        let hash_part = key
            .split('_')
            .next_back()
            .expect("cache key should have underscore-separated parts");
        assert_eq!(hash_part.len(), 12);
    }

    #[test]
    fn human_readable_prefix_basic() {
        let prefix = human_readable_prefix("https://docs.rs/tokio/latest/tokio/");
        assert_eq!(prefix, "docs.rs_tokio_latest_tokio");
    }

    #[test]
    fn human_readable_prefix_truncation() {
        let long_url = "https://example.com/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z";
        let prefix = human_readable_prefix(long_url);
        assert!(prefix.len() <= 90); // some buffer for segment boundaries
    }

    #[test]
    fn human_readable_prefix_excludes_query_and_fragment() {
        let prefix = human_readable_prefix("https://example.com/path?q=foo&bar=baz#section");
        // Query and fragment are stripped by URL parsing; only path segments appear
        assert!(!prefix.contains('?'));
        assert!(!prefix.contains('&'));
        assert!(!prefix.contains('#'));
    }

    #[test]
    fn mime_to_extension_known_types() {
        assert_eq!(mime_to_extension("image/png"), "png");
        assert_eq!(mime_to_extension("image/jpeg"), "jpg");
        assert_eq!(mime_to_extension("image/svg+xml"), "svg");
        assert_eq!(mime_to_extension("image/webp"), "webp");
    }

    #[test]
    fn mime_to_extension_unknown() {
        assert_eq!(mime_to_extension("image/x-custom"), "bin");
    }

    #[test]
    fn output_filename_html_raw() {
        assert_eq!(
            output_filename("text/html", "https://example.com", true),
            "index.html"
        );
    }

    #[test]
    fn output_filename_json() {
        assert_eq!(
            output_filename("application/json", "https://api.example.com/data", false),
            "index.json"
        );
    }

    #[test]
    fn output_filename_pdf() {
        assert_eq!(
            output_filename("application/pdf", "https://example.com/doc.pdf", false),
            "index.pdf"
        );
    }

    #[test]
    fn output_filename_image() {
        assert_eq!(
            output_filename("image/png", "https://example.com/photo.png", false),
            "photo.png"
        );
    }

    #[test]
    fn output_filename_image_no_extension() {
        assert_eq!(
            output_filename("image/png", "https://example.com/images/avatar", false),
            "image.png"
        );
    }

    #[test]
    fn output_filename_strips_leading_underscores_image() {
        assert_eq!(
            output_filename("image/png", "https://example.com/_hidden.png", false),
            "hidden.png"
        );
    }

    #[test]
    fn output_filename_strips_leading_underscores_unknown() {
        assert_eq!(
            output_filename(
                "application/octet-stream",
                "https://example.com/__data.bin",
                false
            ),
            "data.bin"
        );
    }

    #[test]
    fn output_filename_cache_policy_collision_avoided() {
        assert_eq!(
            output_filename(
                "application/json",
                "https://example.com/_cache_policy.json",
                false
            ),
            "index.json"
        );
        assert_eq!(
            output_filename(
                "application/octet-stream",
                "https://example.com/_cache_policy.json",
                false
            ),
            "cache_policy.json"
        );
    }

    #[test]
    fn sanitize_filename_strips_underscores() {
        assert_eq!(sanitize_filename("_cache_policy.json"), "cache_policy.json");
        assert_eq!(sanitize_filename("__double.txt"), "double.txt");
    }

    #[test]
    fn sanitize_filename_preserves_normal_names() {
        assert_eq!(sanitize_filename("photo.png"), "photo.png");
        assert_eq!(sanitize_filename("index.html"), "index.html");
    }

    #[test]
    fn sanitize_filename_all_underscores() {
        assert_eq!(sanitize_filename("___"), "file");
    }

    #[test]
    fn image_collector_resolves_absolute_urls() {
        let mut collector = ImageUrlCollector::new("https://example.com/page");
        let img = ImageObject::new("https://cdn.example.com/photo.png".into());
        collector.check_image(&img);
        assert_eq!(collector.urls.len(), 1);
        assert_eq!(collector.urls[0].0, "https://cdn.example.com/photo.png");
        assert_eq!(collector.urls[0].1, "https://cdn.example.com/photo.png");
    }

    #[test]
    fn image_collector_resolves_relative_urls() {
        let mut collector = ImageUrlCollector::new("https://example.com/docs/page.html");
        let img = ImageObject::new("images/photo.png".into());
        collector.check_image(&img);
        assert_eq!(collector.urls.len(), 1);
        assert_eq!(collector.urls[0].0, "images/photo.png");
        assert_eq!(
            collector.urls[0].1,
            "https://example.com/docs/images/photo.png"
        );
    }

    #[test]
    fn image_collector_resolves_root_relative_urls() {
        let mut collector = ImageUrlCollector::new("https://example.com/docs/page.html");
        let img = ImageObject::new("/assets/logo.svg".into());
        collector.check_image(&img);
        assert_eq!(collector.urls.len(), 1);
        assert_eq!(collector.urls[0].0, "/assets/logo.svg");
        assert_eq!(collector.urls[0].1, "https://example.com/assets/logo.svg");
    }

    #[test]
    fn image_collector_skips_data_uris() {
        let mut collector = ImageUrlCollector::new("https://example.com/page");
        let img = ImageObject::new("data:image/png;base64,abc123".into());
        collector.check_image(&img);
        assert!(collector.urls.is_empty());
    }

    #[test]
    fn image_collector_deduplicates() {
        let mut collector = ImageUrlCollector::new("https://example.com/page");
        let img1 = ImageObject::new("/logo.png".into());
        let img2 = ImageObject::new("/logo.png".into());
        collector.check_image(&img1);
        collector.check_image(&img2);
        assert_eq!(collector.urls.len(), 1);
    }

    #[test]
    fn format_duration_seconds() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
    }

    #[test]
    fn format_duration_minutes() {
        assert_eq!(format_duration(Duration::from_secs(300)), "5m");
    }

    #[test]
    fn format_duration_hours() {
        assert_eq!(format_duration(Duration::from_secs(7200)), "2h");
    }

    #[test]
    fn format_bytes_small() {
        assert_eq!(format_bytes(500), "500B");
    }

    #[test]
    fn format_bytes_kb() {
        assert_eq!(format_bytes(5120), "5.0KB");
    }

    #[test]
    fn format_bytes_mb() {
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0MB");
    }
}
