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

## Limits

- Uploads are capped at 25 MiB.
- Conversions time out after 60 seconds.
- Each request must provide exactly one input: `file` or `url`.
- Remote input is limited to arXiv, bioRxiv, medRxiv, and PMC identifiers and
  URLs. Generic URL fetching is intentionally not supported.
- The Cloudflare Worker rate-limits public conversions per client IP.

## Concurrency

Axum and Tokio handle requests concurrently inside each container process. There
is no global conversion queue in the Rust server. The Cloudflare Worker can also
route requests across multiple `ConvertBackend` container instances.

This means multiple conversions can run at the same time. The Worker rate
limiter controls public request volume, but a future hardening step should add a
small in-process concurrency limit around `/api/convert` to cap CPU and memory
pressure per container.

## File Isolation And Retention

The server does not intentionally retain uploaded files or converted outputs.

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

## Verification

Run the narrow checks after changes:

```sh
cargo fmt -p stencila-convert-server
cargo clippy --fix --allow-dirty --all-targets -p stencila-convert-server
cargo test -p stencila-convert-server
```
