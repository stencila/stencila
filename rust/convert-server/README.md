# Stencila Convert Server

`stencila-convert-server` is the Rust backend for the public converter at
`convert.stencila.dev`. It exposes a small Axum HTTP API and reuses Stencila's
in-process codec dispatch for conversions.

## Local Development

Run the backend and local UI together:

```sh
cargo run -p stencila-convert-server
```

Then open:

```text
http://127.0.0.1:8080/
```

In debug builds, the server serves static UI files directly from
`workers/convert/public` with `Cache-Control: no-store`, so changes to
`index.html`, `app.js`, and `styles.css` are picked up by browser reload.

In production, the Cloudflare Worker serves the same static assets from Workers
Static Assets and proxies only `/api/*` requests to the containerized Rust
backend.

## Public API

- `GET /api/health` returns `{ "ok": true, "version": "<stencila version>" }`.
- `GET /api/isalive` returns `true` as plain text, matching GROBID's liveness
  response shape for orchestration tools.
- `GET /api/formats` returns runtime-supported formats and their direction
  support. The optional `from` and `to` query parameters accept comma-separated
  format names and restrict the formats advertised for each direction, for
  example `/api/formats?from=markdown,docx&to=json,html`.
- `POST /api/convert` accepts one uploaded file or a supported repository
  identifier/URL and returns the converted document inline or as an attachment.
  Supported repositories are arXiv, bioRxiv, medRxiv, and PubMed Central (PMC).
  Both the legacy `10.1101` and current `10.64898` OpenRxiv DOI prefixes are
  accepted.

When an attachment conversion produces sidecar files, such as extracted images
alongside Markdown, the response is a ZIP archive containing the primary output
and all generated files.

The OXA codec is exposed as two concrete output formats: OXA JSON (`.oxa.json`)
and OXA YAML (`.oxa.yaml`). Both are routed through the same codec, but the file
extension determines the serialized flavor.

## GROBID-Shaped JATS API

- `POST` or `PUT /api/processFulltextDocument` accepts a `multipart/form-data`
  request with the document to convert in the `input` part and returns JATS XML.
  The legacy `file` field used by ScienceBeam is also accepted.

This route mirrors the request shape of
[GROBID's](https://grobid.readthedocs.io) fulltext endpoint, but it is not a
general GROBID replacement. GROBID returns TEI XML by default; Stencila returns
JATS because it does not currently have a TEI encoder. Clients that require TEI,
including ScienceBeam's default benchmark workflow, are therefore not
compatible with this route.

- GROBID's other form fields, such as `includeRawAffiliations` and
  `includeRawCitations`, are accepted and ignored.
- A request with neither an `input` nor `file` upload, or with more than one
  file, is rejected with `400`. Filename-less text fields do not count as an
  upload, and unrelated file field names are not accepted as the input.
- The response is always inline, with `Content-Type: application/xml;
  charset=utf-8`, never a ZIP attachment. Any sidecar files a conversion might
  otherwise produce, such as extracted images, are discarded.
- `Accept: application/xml`, `text/xml`, `application/vnd.jats+xml`, `*/*`, and
  a missing `Accept` header all return JATS. Any other explicitly requested type,
  or an allowed type with quality `q=0`, returns `406`.
- GROBID only accepts PDF in `input`. This server also accepts any other format
  it can decode, choosing the format from the uploaded filename and falling back
  to PDF when it is not recognized.

Errors use the same JSON error body as `/api/convert`. In particular, exceeding
the conversion time limit returns `504` with the code `conversion_timeout`.

## Limits

- Uploads are capped at 25 MiB by default.
- Conversions time out after 60 seconds by default.
- Each request must provide exactly one input: `file` or `url`.
- Remote input is limited to arXiv, bioRxiv, medRxiv, and PMC identifiers and
  URLs. Generic URL fetching is intentionally not supported.
- The Cloudflare Worker rate-limits public conversions per client IP.

## Configuration

Every setting keeps its previous hard-coded value as the default, so the public
converter is unaffected unless an operator opts in.

| Environment variable                | Default     | Effect                                                     |
| ----------------------------------- | ----------- | ---------------------------------------------------------- |
| `STENCILA_CONVERT_TIMEOUT_SECONDS`  | `60`        | Time limit for a single conversion                         |
| `STENCILA_CONVERT_MAX_UPLOAD_MB`    | `25`        | Maximum uploaded file size; the request limit tracks it     |
| `STENCILA_CONVERT_MAX_CONCURRENCY`  | _unlimited_ | Maximum conversions running at the same time               |
| `STENCILA_CONVERT_ARTIFACTS_DIR`    | _unset_     | Directory in which to retain decoding artifacts (see below) |
| `STENCILA_CONVERT_PORT`             | `8080`      | Port to bind to                                            |

Values that are empty or invalid are logged and ignored, falling back to the
default. If the configured artifacts directory can not be created or entered,
the error is logged and artifact retention is disabled.

## Concurrency

Axum and Tokio handle requests concurrently inside each container process. The
Cloudflare Worker can also route requests across multiple `ConvertBackend`
container instances.

By default there is no in-process limit, so multiple conversions can run at the
same time and the Worker rate limiter is the only backpressure on public request
volume. Setting `STENCILA_CONVERT_MAX_CONCURRENCY` adds a semaphore around the
conversion path in both `/api/convert` and `/api/processFulltextDocument`, to cap
CPU and memory pressure per container. Requests beyond the limit wait for a
permit rather than being rejected, and the conversion time limit is applied after
a permit is acquired.

## File Isolation And Retention

By default, and therefore for the public converter, the server does not
intentionally retain uploaded files or converted outputs.

- Uploaded multipart bodies are parsed in memory by Axum up to the configured
  request limit.
  
- Uploaded files are written into a fresh request-local `tempfile::TempDir`.

- Download-mode outputs are written into another fresh request-local temp
  directory, read back into the response, and then deleted when the temp
  directory is dropped.

- Inline text outputs are returned from memory.

- Filenames are sanitized before they are used on disk or in response headers.

- Remote decoding is called with `ignore_artifacts: true` and `no_artifacts:
  true`, so persistent Stencila artifact caches are not created for public
  conversions.

Because each request gets its own temporary directories, normal concurrent
requests should not see each other's files. If the process or container crashes
mid-request, temporary files may remain on the container's ephemeral filesystem
until the container is restarted or destroyed. They are not written to durable
storage by this service.

### Artifact Retention

Setting `STENCILA_CONVERT_ARTIFACTS_DIR` deliberately changes this. When it is
set:

- Decoding runs with `ignore_artifacts: false` and `no_artifacts: false`, so
  codecs create and reuse artifacts such as OCR output for PDFs.
- The server creates `<dir>/.stencila/artifacts` at startup and makes `<dir>`
  the process working directory, which is how codecs locate that tree.
- Artifacts are keyed by a hash of the **input content**, not its path, so
  re-uploading the same document is a cache hit even though each upload lands in
  a different temporary directory.

This means derived output from uploaded documents is retained on disk, across
requests and across restarts if the directory is a mounted volume. It is
therefore off by default and **must not** be enabled for the public converter,
whose retention behaviour is described above.

Its purpose is repeated conversion of a fixed corpus: PDF decoding calls the
paid Mistral OCR API using the model selected by the PDF codec (requiring
`MISTRAL_API_KEY`), and caching means a corpus is only OCR'd once no matter how
many times it is re-converted.

## Verification

Run the narrow checks after changes:

```sh
cargo fmt -p stencila-convert-server
cargo clippy --fix --allow-dirty --all-targets -p stencila-convert-server
cargo test -p stencila-convert-server
```
