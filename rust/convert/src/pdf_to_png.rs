//! PDF to PNG conversion using PDF.js in headless Chrome.

use std::{
    fs,
    io::Write,
    path::Path,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use eyre::{Context as _, Result, bail, eyre};
use headless_chrome::protocol::cdp::Page;
use serde_json::json;
use sha2::{Digest, Sha256};
use stencila_dirs::{DirType, get_app_dir};
use tempfile::Builder;

use crate::html_to_png::with_browser_tab;

const PDFJS_VERSION: &str = "5.4.394";
const PDFJS_MODULE_SHA256: &str =
    "b33deb12e2f6c0c8dfb1e30c87f1f9a3f03dc4c56d330264dbb86edc21fac905";
const PDFJS_WORKER_SHA256: &str =
    "cee92175925a2a7a43f3450a99e583fcbecc2685509e1dc270a03a186861592f";

const PDFJS_MODULE_URL: &str = "https://cdn.jsdelivr.net/npm/pdfjs-dist@5.4.394/build/pdf.mjs";
const PDFJS_WORKER_URL: &str =
    "https://cdn.jsdelivr.net/npm/pdfjs-dist@5.4.394/build/pdf.worker.mjs";
const DEFAULT_PDF_TO_PNG_TIMEOUT: Duration = Duration::from_secs(15);
const PDFJS_CONNECT_TIMEOUT_LIMIT: Duration = Duration::from_secs(10);
static PDFJS_CACHE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Environment variable controlling whether PDF.js assets may be downloaded.
///
/// Set to `never`, `off`, `false`, or `0` to require an existing verified
/// cache entry and skip CDN fetches.
const PDFJS_CDN_ENV: &str = "STENCILA_PDFJS_CDN";

/// Options for rendering a PDF page to a PNG.
#[derive(Debug, Clone, Copy)]
pub struct PdfToPngOptions {
    /// One-based page number to render.
    pub page: u32,

    /// Maximum rendered page width in CSS pixels.
    pub max_width: u32,

    /// Maximum time to spend fetching missing renderer assets and rendering.
    pub timeout: Duration,
}

impl Default for PdfToPngOptions {
    fn default() -> Self {
        Self {
            page: 1,
            max_width: 512,
            timeout: DEFAULT_PDF_TO_PNG_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PdfJsAsset {
    file_name: &'static str,
    url: &'static str,
    sha256: &'static str,
}

const PDFJS_ASSETS: [PdfJsAsset; 2] = [
    PdfJsAsset {
        file_name: "pdf.mjs",
        url: PDFJS_MODULE_URL,
        sha256: PDFJS_MODULE_SHA256,
    },
    PdfJsAsset {
        file_name: "pdf.worker.mjs",
        url: PDFJS_WORKER_URL,
        sha256: PDFJS_WORKER_SHA256,
    },
];

/// Render the first page of a PDF to PNG bytes.
///
/// The PDF.js runtime is downloaded from a pinned CDN URL into the Stencila app
/// cache on first use, then verified by SHA-256 before every render.
///
/// # Errors
///
/// Returns an error if PDF.js cannot be cached or verified, Chrome/Chromium
/// cannot render the PDF, or the rendered canvas does not produce a PNG data
/// URL.
pub fn pdf_to_png_bytes(path: &Path, options: PdfToPngOptions) -> Result<Vec<u8>> {
    if !path.is_file() {
        bail!("PDF does not exist or is not a file: {}", path.display());
    }

    let timeout = normalize_timeout(options.timeout);
    let deadline = Instant::now() + timeout;
    let pdf =
        fs::read(path).wrap_err_with(|| format!("failed to read PDF at {}", path.display()))?;
    let assets = ensure_pdfjs_assets(deadline)?;
    render_pdf_page(&pdf, &assets, options, remaining_timeout(deadline)?)
}

/// Render the first page of a PDF to a PNG file.
///
/// # Errors
///
/// Returns an error if rendering fails or the PNG file cannot be written.
pub fn pdf_to_png_file(path: &Path, output: &Path, options: PdfToPngOptions) -> Result<()> {
    let bytes = pdf_to_png_bytes(path, options)?;
    fs::write(output, bytes)
        .wrap_err_with(|| format!("failed to write PDF thumbnail to {}", output.display()))
}

#[derive(Debug)]
struct PdfJsAssets {
    module: Vec<u8>,
    worker: Vec<u8>,
}

fn ensure_pdfjs_assets(deadline: Instant) -> Result<PdfJsAssets> {
    let cache_dir = get_app_dir(DirType::Cache, true)
        .wrap_err("failed to locate Stencila cache directory")?
        .join("pdfjs")
        .join(PDFJS_VERSION);
    fs::create_dir_all(&cache_dir)
        .wrap_err_with(|| format!("failed to create PDF.js cache at {}", cache_dir.display()))?;

    let _cache_lock = PDFJS_CACHE_LOCK
        .lock()
        .map_err(|error| eyre!("failed to acquire PDF.js cache lock: {error}"))?;

    Ok(PdfJsAssets {
        module: ensure_pdfjs_asset(&cache_dir, PDFJS_ASSETS[0], deadline)?,
        worker: ensure_pdfjs_asset(&cache_dir, PDFJS_ASSETS[1], deadline)?,
    })
}

fn ensure_pdfjs_asset(cache_dir: &Path, asset: PdfJsAsset, deadline: Instant) -> Result<Vec<u8>> {
    let path = cache_dir.join(asset.file_name);
    if let Ok(bytes) = fs::read(&path)
        && sha256_hex(&bytes) == asset.sha256
    {
        return Ok(bytes);
    }

    if cdn_disabled() {
        bail!(
            "verified PDF.js asset is not cached and {PDFJS_CDN_ENV} disables CDN fetches: {}",
            path.display()
        );
    }

    let timeout = remaining_timeout(deadline)?;
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(timeout.min(PDFJS_CONNECT_TIMEOUT_LIMIT))
        .timeout(timeout)
        .build()
        .wrap_err("failed to create PDF.js asset HTTP client")?;

    let bytes = client
        .get(asset.url)
        .send()
        .wrap_err_with(|| format!("failed to fetch PDF.js asset from {}", asset.url))?
        .error_for_status()
        .wrap_err_with(|| format!("failed to fetch PDF.js asset from {}", asset.url))?
        .bytes()
        .wrap_err_with(|| format!("failed to read PDF.js asset from {}", asset.url))?;
    let bytes = bytes.to_vec();

    let actual = sha256_hex(&bytes);
    if actual != asset.sha256 {
        bail!(
            "PDF.js asset checksum mismatch for {}: expected {}, got {}",
            asset.file_name,
            asset.sha256,
            actual
        );
    }

    let tmp_prefix = format!("{}-", asset.file_name);
    let mut tmp = Builder::new()
        .prefix(&tmp_prefix)
        .tempfile_in(cache_dir)
        .wrap_err_with(|| {
            format!(
                "failed to create temporary PDF.js asset file in {}",
                cache_dir.display()
            )
        })?;
    tmp.write_all(&bytes)
        .wrap_err_with(|| format!("failed to write PDF.js asset to {}", tmp.path().display()))?;
    tmp.flush()
        .wrap_err_with(|| format!("failed to flush PDF.js asset at {}", tmp.path().display()))?;
    tmp.persist(&path)
        .map(|_| ())
        .wrap_err_with(|| format!("failed to cache PDF.js asset at {}", path.display()))?;

    Ok(bytes)
}

fn render_pdf_page(
    pdf: &[u8],
    assets: &PdfJsAssets,
    options: PdfToPngOptions,
    timeout: Duration,
) -> Result<Vec<u8>> {
    let page = options.page.max(1);
    let max_width = options.max_width.max(1);
    let html = render_html(
        &BASE64.encode(&assets.module),
        &BASE64.encode(&assets.worker),
        &BASE64.encode(pdf),
    );

    let png_data_url = with_browser_tab(|tab| {
        tab.set_default_timeout(timeout);
        tab.call_method(Page::SetLifecycleEventsEnabled { enabled: true })
            .map_err(|error| eyre!("failed to enable browser lifecycle events: {error}"))?;

        let frame_tree = tab
            .call_method(Page::GetFrameTree(None))
            .map_err(|error| eyre!("failed to get browser frame tree: {error}"))?;
        tab.call_method(Page::SetDocumentContent {
            frame_id: frame_tree.frame_tree.frame.id,
            html,
        })
        .map_err(|error| eyre!("failed to load PDF thumbnail renderer: {error}"))?;

        let script = render_script(page, max_width, timeout);
        let result = tab
            .evaluate(&script, true)
            .map_err(|error| eyre!("failed to render PDF thumbnail: {error}"))?;
        result
            .value
            .and_then(|value| value.as_str().map(ToString::to_string))
            .ok_or_else(|| eyre!("PDF thumbnail renderer did not return a PNG data URL"))
    })?;

    decode_png_data_url(&png_data_url)
}

fn render_html(pdfjs_module: &str, pdfjs_worker: &str, pdf: &str) -> String {
    format!(
        r#"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    html,
    body {{
      margin: 0;
      padding: 0;
      background: white;
    }}
    canvas {{
      display: block;
    }}
  </style>
</head>
<body>
  <canvas id="pdf-page"></canvas>
  <script>
    const decodeBase64 = (base64) => {{
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let index = 0; index < binary.length; index += 1) {{
        bytes[index] = binary.charCodeAt(index);
      }}
      return bytes;
    }};

    const scriptBlobUrl = (base64) => URL.createObjectURL(
      new Blob([decodeBase64(base64)], {{ type: "text/javascript" }})
    );

    (async () => {{
      try {{
        const moduleUrl = scriptBlobUrl({pdfjs_module});
        const workerUrl = scriptBlobUrl({pdfjs_worker});
        const pdfData = decodeBase64({pdf});
        const pdfjsLib = await import(moduleUrl);

        pdfjsLib.GlobalWorkerOptions.workerSrc = workerUrl;

        window.__stencilaPdfThumbnailCleanup = () => {{
          URL.revokeObjectURL(moduleUrl);
          URL.revokeObjectURL(workerUrl);
        }};

        window.__stencilaRenderPdfThumbnail = async (pageNumber, maxWidth) => {{
          const loadingTask = pdfjsLib.getDocument({{
            data: pdfData.slice(),
            useSystemFonts: true,
          }});
          const pdfDocument = await loadingTask.promise;
          try {{
            const page = await pdfDocument.getPage(pageNumber);
            const baseViewport = page.getViewport({{ scale: 1 }});
            const scale = Math.min(1, maxWidth / baseViewport.width);
            const viewport = page.getViewport({{ scale }});
            const canvas = document.getElementById("pdf-page");
            const context = canvas.getContext("2d", {{ alpha: false }});
            if (!context) {{
              throw new Error("Could not create PDF thumbnail canvas context");
            }}
            canvas.width = Math.max(1, Math.floor(viewport.width));
            canvas.height = Math.max(1, Math.floor(viewport.height));
            context.fillStyle = "white";
            context.fillRect(0, 0, canvas.width, canvas.height);
            await page.render({{
              canvasContext: context,
              viewport,
              background: "white",
            }}).promise;
            return canvas.toDataURL("image/png");
          }} finally {{
            await pdfDocument.destroy();
          }}
        }};
      }} catch (error) {{
        window.__stencilaPdfThumbnailSetupError =
          error?.stack || error?.message || String(error);
      }}
    }})();
  </script>
</body>
</html>"#,
        pdfjs_module = json!(pdfjs_module),
        pdfjs_worker = json!(pdfjs_worker),
        pdf = json!(pdf)
    )
}

fn render_script(page: u32, max_width: u32, timeout: Duration) -> String {
    format!(
        r#"(async () => {{
  const timeoutMs = {};
  const wait = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));
  const render = (async () => {{
  const started = Date.now();
  while (typeof window.__stencilaRenderPdfThumbnail !== "function") {{
    if (window.__stencilaPdfThumbnailSetupError) {{
      throw new Error(`Failed to load PDF.js renderer: ${{window.__stencilaPdfThumbnailSetupError}}`);
    }}
    if (Date.now() - started > timeoutMs) {{
      throw new Error("Timed out loading PDF.js renderer");
    }}
    await wait(50);
  }}
    try {{
      return await window.__stencilaRenderPdfThumbnail({}, {});
    }} finally {{
      if (typeof window.__stencilaPdfThumbnailCleanup === "function") {{
        window.__stencilaPdfThumbnailCleanup();
      }}
    }}
  }})();
  const timeout = new Promise((_, reject) => {{
    setTimeout(() => reject(new Error("Timed out rendering PDF page")), timeoutMs);
  }});
  return await Promise.race([render, timeout]);
}})()"#,
        timeout.as_millis().max(1),
        page,
        max_width
    )
}

fn decode_png_data_url(data_url: &str) -> Result<Vec<u8>> {
    let Some(data) = data_url.strip_prefix("data:image/png;base64,") else {
        bail!("PDF thumbnail renderer returned a non-PNG data URL");
    };

    BASE64
        .decode(data)
        .wrap_err("failed to decode PDF thumbnail PNG data URL")
}

fn cdn_disabled() -> bool {
    std::env::var(PDFJS_CDN_ENV)
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "never" | "off" | "false" | "0"
            )
        })
        .unwrap_or(false)
}

fn normalize_timeout(timeout: Duration) -> Duration {
    timeout.max(Duration::from_millis(1))
}

fn remaining_timeout(deadline: Instant) -> Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|timeout| !timeout.is_zero())
        .ok_or_else(|| eyre!("timed out rendering PDF thumbnail"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(bytes))
}

fn lower_hex(bytes: &[u8]) -> String {
    const CHARS: &[u8; 16] = b"0123456789abcdef";

    let mut hex = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        hex.push(char::from(CHARS[usize::from(byte >> 4)]));
        hex.push(char::from(CHARS[usize::from(byte & 0x0f)]));
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_is_stable() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn decode_png_data_url_rejects_non_png() {
        let error = decode_png_data_url("data:image/jpeg;base64,abc")
            .expect_err("non-PNG data URLs should be rejected");

        assert!(
            error
                .to_string()
                .contains("renderer returned a non-PNG data URL")
        );
    }

    #[test]
    fn pdfjs_asset_names_are_stable() {
        assert_eq!(PDFJS_ASSETS[0].file_name, "pdf.mjs");
        assert_eq!(PDFJS_ASSETS[1].file_name, "pdf.worker.mjs");
    }

    #[test]
    fn render_html_uses_injected_blobs() {
        let html = render_html("module", "worker", "pdf");

        assert!(html.contains("URL.createObjectURL"));
        assert!(html.contains("data: pdfData.slice()"));
        assert!(!html.contains("file://"));
    }

    #[test]
    #[ignore = "requires Chrome/Chromium plus PDF.js cache or network access"]
    fn render_sample_pdf_to_png() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../content-credentials/tests/fixtures/sample.pdf");
        let bytes = pdf_to_png_bytes(
            &path,
            PdfToPngOptions {
                page: 1,
                max_width: 256,
                ..Default::default()
            },
        )
        .expect("render PDF thumbnail");

        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
}
