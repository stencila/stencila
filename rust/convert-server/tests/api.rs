use axum::{
    body::{Body, to_bytes},
    http::{
        Method, Request, StatusCode,
        header::{ACCEPT, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE},
    },
};
use bytes::Bytes;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};
use stencila_convert_server::{ServerConfig, app, app_with_config};
use tokio_stream::wrappers::ReceiverStream;
use tower::ServiceExt;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const MIB: usize = 1024 * 1024;
type PrivateFrames = Vec<(Value, Vec<u8>)>;

fn decode_hex_fixture(value: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let value = value.trim();
    if !value.len().is_multiple_of(2) {
        return Err("hex fixture has an odd length".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| Ok(u8::from_str_radix(&value[index..index + 2], 16)?))
        .collect()
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn serves_development_index_html() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::builder().uri("/").body(Body::empty())?;
    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE),
        Some(&"text/html".parse()?)
    );
    assert_eq!(
        response.headers().get(CACHE_CONTROL),
        Some(&"no-store".parse()?)
    );

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let html = std::str::from_utf8(&bytes)?;
    assert!(html.contains("Stencila Convert"));

    Ok(())
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn serves_development_asset() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::builder().uri("/app.js").body(Body::empty())?;
    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let script = std::str::from_utf8(&bytes)?;
    assert!(script.contains("loadFormats"));

    Ok(())
}

#[tokio::test]
async fn converts_multipart_markdown_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-test-boundary";
    let body = [
        format!("--{boundary}\r\n"),
        "Content-Disposition: form-data; name=\"to\"\r\n\r\n".to_string(),
        "json\r\n".to_string(),
        format!("--{boundary}\r\n"),
        "Content-Disposition: form-data; name=\"file\"; filename=\"paper.md\"\r\n".to_string(),
        "Content-Type: text/markdown\r\n\r\n".to_string(),
        "# Test\n\nA paragraph.\r\n".to_string(),
        format!("--{boundary}--\r\n"),
    ]
    .concat();

    let request = Request::builder()
        .method("POST")
        .uri("/api/convert")
        .header(
            CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))?;

    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("X-Stencila-Output-Filename"),
        Some(&"paper.json".parse()?)
    );

    let bytes = to_bytes(response.into_body(), 2 * MIB).await?;
    let json: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(json["type"], "Article");

    Ok(())
}

#[tokio::test]
async fn filters_formats_using_direction_allowlists() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::builder()
        .uri("/api/formats?from=markdown&to=json")
        .body(Body::empty())?;
    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let formats: Value = serde_json::from_slice(&bytes)?;
    let formats = formats
        .as_array()
        .ok_or("formats response should be an array")?;

    assert_eq!(formats.len(), 2);
    assert!(
        formats.iter().any(|format| format["name"] == "md"
            && format["from"] == true
            && format["to"] == false)
    );
    assert!(formats.iter().any(|format| {
        format["name"] == "json" && format["from"] == false && format["to"] == true
    }));
    assert!(formats.iter().all(|format| format["name"] != "directory"));

    Ok(())
}

#[tokio::test]
async fn archives_downloads_with_sidecar_files() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-archive-test-boundary";
    let body = [
        format!("--{boundary}\r\n"),
        "Content-Disposition: form-data; name=\"to\"\r\n\r\n".to_string(),
        "markdown\r\n".to_string(),
        format!("--{boundary}\r\n"),
        "Content-Disposition: form-data; name=\"mode\"\r\n\r\n".to_string(),
        "download\r\n".to_string(),
        format!("--{boundary}\r\n"),
        "Content-Disposition: form-data; name=\"file\"; filename=\"paper.md\"\r\n"
            .to_string(),
        "Content-Type: text/markdown\r\n\r\n".to_string(),
        "![pixel](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=)\r\n".to_string(),
        format!("--{boundary}--\r\n"),
    ]
    .concat();

    let request = Request::builder()
        .method("POST")
        .uri("/api/convert")
        .header(
            CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))?;

    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE),
        Some(&"application/zip".parse()?)
    );
    assert_eq!(
        response.headers().get("X-Stencila-Output-Filename"),
        Some(&"paper.zip".parse()?)
    );

    let bytes = to_bytes(response.into_body(), 2 * MIB).await?;
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let mut markdown = String::new();
    archive.by_name("paper.md")?.read_to_string(&mut markdown)?;
    assert!(markdown.contains("paper.media/"));
    assert!(
        (0..archive.len())
            .filter_map(|index| archive
                .by_index(index)
                .ok()
                .map(|file| file.name().to_string()))
            .any(|name| name.starts_with("paper.media/") && name.ends_with(".png"))
    );

    Ok(())
}

/// Build a GROBID-style `multipart/form-data` body
///
/// `files` are `(field name, filename, content)`; `fields` are plain text form
/// fields.
fn grobid_body(boundary: &str, files: &[(&str, &str, &str)], fields: &[(&str, &str)]) -> String {
    let mut body = String::new();
    for (name, filename, content) in files {
        body.push_str(&format!("--{boundary}\r\n"));
        body.push_str(&format!(
            "Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n"
        ));
        body.push_str("Content-Type: application/pdf\r\n\r\n");
        body.push_str(content);
        body.push_str("\r\n");
    }
    for (name, value) in fields {
        body.push_str(&format!("--{boundary}\r\n"));
        body.push_str(&format!(
            "Content-Disposition: form-data; name=\"{name}\"\r\n\r\n"
        ));
        body.push_str(value);
        body.push_str("\r\n");
    }
    body.push_str(&format!("--{boundary}--\r\n"));
    body
}

fn grobid_request(
    body: String,
    boundary: &str,
    accept: Option<&str>,
) -> Result<Request<Body>, Box<dyn std::error::Error>> {
    grobid_request_with_method(Method::POST, body, boundary, accept)
}

fn grobid_request_with_method(
    method: Method,
    body: String,
    boundary: &str,
    accept: Option<&str>,
) -> Result<Request<Body>, Box<dyn std::error::Error>> {
    let mut builder = Request::builder()
        .method(method)
        .uri("/api/processFulltextDocument")
        .header(
            CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        );
    if let Some(accept) = accept {
        builder = builder.header(ACCEPT, accept);
    }
    Ok(builder.body(Body::from(body))?)
}

#[tokio::test]
async fn converts_fulltext_document_to_jats() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-grobid-test-boundary";
    let body = grobid_body(
        boundary,
        &[("input", "paper.md", "# Test\n\nA paragraph.")],
        // GROBID flags, which are accepted and ignored
        &[
            ("includeRawAffiliations", "1"),
            ("includeRawCitations", "1"),
            ("consolidateHeader", "0"),
        ],
    );

    // A missing `Accept` header defaults to the route's JATS representation
    let response = app().oneshot(grobid_request(body, boundary, None)?).await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE),
        Some(&"application/xml; charset=utf-8".parse()?)
    );

    let bytes = to_bytes(response.into_body(), 2 * MIB).await?;
    let xml = std::str::from_utf8(&bytes)?;
    assert!(xml.contains("<article"), "unexpected body: {xml}");
    assert!(xml.contains("A paragraph."));

    Ok(())
}

#[tokio::test]
async fn accepts_xml_content_negotiation() -> Result<(), Box<dyn std::error::Error>> {
    for accept in [
        "*/*",
        "application/xml",
        "text/xml",
        "application/vnd.jats+xml",
        "text/html, application/xml;q=0.9",
        "application/json, */*;q=0.5",
    ] {
        let boundary = "stencila-grobid-accept-boundary";
        let body = grobid_body(boundary, &[("input", "paper.md", "# Test")], &[]);
        let response = app()
            .oneshot(grobid_request(body, boundary, Some(accept))?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK, "for accept: {accept}");
    }

    Ok(())
}

#[tokio::test]
async fn rejects_unsupported_accept_header() -> Result<(), Box<dyn std::error::Error>> {
    for accept in [
        "application/json",
        "application/*",
        "text/*",
        "application/jats+xml",
        "text/jats+xml",
        "application/tei+xml",
        "application/xml;q=0",
        "application/xml;q=0, */*",
        "application/xml;q=invalid",
        "application/xml;q=2",
        "*/*;q=0",
    ] {
        let boundary = "stencila-grobid-406-boundary";
        let body = grobid_body(boundary, &[("input", "paper.md", "# Test")], &[]);

        let response = app()
            .oneshot(grobid_request(body, boundary, Some(accept))?)
            .await?;
        assert_eq!(
            response.status(),
            StatusCode::NOT_ACCEPTABLE,
            "for accept: {accept}"
        );

        let bytes = to_bytes(response.into_body(), MIB).await?;
        let json: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(json["error"]["code"], "not_acceptable");
    }

    Ok(())
}

#[tokio::test]
async fn accepts_fulltext_document_over_put() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-grobid-put-boundary";
    let body = grobid_body(boundary, &[("input", "paper.md", "# Test")], &[]);

    let response = app()
        .oneshot(grobid_request_with_method(
            Method::PUT,
            body,
            boundary,
            None,
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn accepts_sciencebeam_file_alias() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-sciencebeam-file-boundary";
    let body = grobid_body(boundary, &[("file", "paper.md", "# Test")], &[]);

    let response = app().oneshot(grobid_request(body, boundary, None)?).await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn rejects_fulltext_document_without_input() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-grobid-missing-boundary";
    let body = grobid_body(boundary, &[], &[("includeRawCitations", "1")]);

    let response = app().oneshot(grobid_request(body, boundary, None)?).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let json: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(json["error"]["code"], "missing_input");

    Ok(())
}

#[tokio::test]
async fn rejects_unrecognized_fulltext_file_field() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-grobid-wrong-file-field-boundary";
    let body = grobid_body(boundary, &[("document", "paper.md", "# Test")], &[]);

    let response = app().oneshot(grobid_request(body, boundary, None)?).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let json: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(json["error"]["code"], "missing_input");

    Ok(())
}

#[tokio::test]
async fn rejects_filename_less_fulltext_input() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-grobid-text-input-boundary";
    let body = grobid_body(boundary, &[], &[("input", "# Test")]);

    let response = app().oneshot(grobid_request(body, boundary, None)?).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let json: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(json["error"]["code"], "missing_input");

    Ok(())
}

#[tokio::test]
async fn rejects_multiple_fulltext_document_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-grobid-multiple-boundary";
    let body = grobid_body(
        boundary,
        &[("input", "one.md", "# One"), ("input", "two.md", "# Two")],
        &[],
    );

    let response = app().oneshot(grobid_request(body, boundary, None)?).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let json: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(json["error"]["code"], "multiple_inputs");

    Ok(())
}

#[tokio::test]
async fn returns_inline_xml_for_documents_with_media() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = "stencila-grobid-media-boundary";
    let body = grobid_body(
        boundary,
        &[(
            "input",
            "paper.md",
            "![pixel](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=)",
        )],
        &[],
    );

    let response = app().oneshot(grobid_request(body, boundary, None)?).await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Sidecar images must not turn the response into a ZIP attachment
    assert_eq!(
        response.headers().get(CONTENT_TYPE),
        Some(&"application/xml; charset=utf-8".parse()?)
    );
    assert_eq!(
        response.headers().get(CONTENT_DISPOSITION),
        Some(&"inline; filename=\"paper.jats.xml\"".parse()?)
    );

    let bytes = to_bytes(response.into_body(), 2 * MIB).await?;
    let xml = std::str::from_utf8(&bytes)?;
    assert!(xml.contains("<article"), "unexpected body: {xml}");

    Ok(())
}

#[tokio::test]
async fn reports_liveness_for_grobid_clients() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::builder().uri("/api/isalive").body(Body::empty())?;
    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), 1024).await?;
    assert_eq!(std::str::from_utf8(&bytes)?, "true");

    Ok(())
}

#[tokio::test]
async fn reports_health_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::builder().uri("/api/health").body(Body::empty())?;
    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let json: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(json["ok"], true);
    assert!(json["version"].is_string());

    Ok(())
}

#[tokio::test]
async fn converts_with_a_configured_concurrency_limit() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig {
        max_concurrency: Some(1),
        ..Default::default()
    };

    let boundary = "stencila-grobid-concurrency-boundary";
    let body = grobid_body(boundary, &[("input", "paper.md", "# Test")], &[]);

    let response = app_with_config(config)
        .oneshot(grobid_request(body, boundary, None)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn rejects_json_unsupported_url() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::builder()
        .method("POST")
        .uri("/api/convert")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"url":"https://example.com/paper.md","to":"json","mode":"inline"}"#,
        ))?;

    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), MIB).await?;
    let json: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(json["error"]["code"], "unsupported_url");

    Ok(())
}

fn private_frame(header: Value, payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let json = serde_json::to_vec(&header)?;
    let mut frame = Vec::with_capacity(4 + json.len() + payload.len());
    frame.extend_from_slice(&(json.len() as u32).to_be_bytes());
    frame.extend_from_slice(&json);
    frame.extend_from_slice(payload);
    Ok(frame)
}

fn private_request(
    from: &str,
    to: &str,
    main_path: &str,
    manuscript_path: Option<&str>,
    files: &[(&str, &[u8])],
) -> Result<Request<Body>, Box<dyn std::error::Error>> {
    let declarations: Vec<_> = files.iter().map(|(path, bytes)| serde_json::json!({
        "path": path,
        "size": bytes.len(),
        "sha256": Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
    })).collect();
    let mut body = private_frame(
        serde_json::json!({
            "kind": "request", "protocol": "conversion.v1", "jobId": "job-1", "attempt": 1,
            "from": from, "to": to,
            "options": {"compact": false, "standalone": false, "media": "sidecar"},
            "mainPath": main_path, "manuscriptPath": manuscript_path, "files": declarations,
            "policy": {"allowModels": false, "allowCodeExecution": false, "allowNetwork": false,
                "supplements": "retain-original"},
            "bounds": {"maxExpandedBytes": 104857600, "maxArchiveEntries": 1000,
                "maxArtifactBytes": 20971520, "maxResultBytes": 104857600, "maxArtifacts": 256}
        }),
        &[],
    )?;
    for (path, bytes) in files {
        for (index, chunk) in bytes.chunks(256 * 1024).enumerate() {
            body.extend(private_frame(
                serde_json::json!({
                    "kind": "file", "path": path, "offset": index * 256 * 1024, "bytes": chunk.len()
                }),
                chunk,
            )?);
        }
    }
    body.extend(private_frame(serde_json::json!({"kind": "end"}), &[])?);
    Ok(Request::builder()
        .method("POST")
        .uri("/private/v1/convert")
        .header(
            CONTENT_TYPE,
            "application/vnd.stencila.conversion.v1+frames",
        )
        .body(Body::from(body))?)
}

fn private_response_frames(mut bytes: &[u8]) -> Result<PrivateFrames, Box<dyn std::error::Error>> {
    let mut frames = Vec::new();
    while !bytes.is_empty() {
        if bytes.len() < 4 {
            return Err("truncated frame length".into());
        }
        let length = u32::from_be_bytes(bytes[..4].try_into()?) as usize;
        bytes = &bytes[4..];
        if bytes.len() < length {
            return Err("truncated frame header".into());
        }
        let header: Value = serde_json::from_slice(&bytes[..length])?;
        bytes = &bytes[length..];
        let payload = header.get("bytes").and_then(Value::as_u64).unwrap_or(0) as usize;
        if bytes.len() < payload {
            return Err("truncated frame payload".into());
        }
        frames.push((header, bytes[..payload].to_vec()));
        bytes = &bytes[payload..];
    }
    Ok(frames)
}

#[tokio::test]
async fn reports_private_build_and_effective_limits() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig {
        max_concurrency: Some(1),
        ..Default::default()
    };
    let response = app_with_config(config)
        .oneshot(
            Request::builder()
                .uri("/private/v1/health")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let health: Value = serde_json::from_slice(&to_bytes(response.into_body(), MIB).await?)?;
    assert_eq!(health["build"]["protocol"], "conversion.v1");
    assert_eq!(health["limits"]["maxConcurrency"], 1);
    assert_eq!(health["limits"]["artifactCacheEnabled"], false);
    assert_eq!(health["inFlight"], 0);
    assert!(health["supportedPairs"].as_array().is_some_and(|pairs| {
        pairs
            .iter()
            .any(|pair| pair["from"] == "markdown" && pair["to"] == "html")
    }));
    Ok(())
}

#[tokio::test]
async fn private_protocol_rejects_legacy_stencila_json_alias()
-> Result<(), Box<dyn std::error::Error>> {
    let response = app()
        .oneshot(private_request(
            "stencila-json",
            "markdown",
            "article.json",
            None,
            &[("article.json", br#"{"type":"Article"}"#)],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    Ok(())
}

#[tokio::test]
async fn privately_converts_generic_codec_formats() -> Result<(), Box<dyn std::error::Error>> {
    let response = app()
        .oneshot(private_request(
            "markdown",
            "html",
            "article.md",
            None,
            &[("article.md", b"# Generic conversion\n")],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
    assert!(frames.iter().any(|(frame, payload)| {
        frame["kind"] == "part"
            && frame["id"] == "primary"
            && String::from_utf8_lossy(payload).contains("Generic conversion")
    }));
    assert!(
        frames
            .iter()
            .any(|(frame, _)| { frame["kind"] == "artifact" && frame["path"] == "document.html" })
    );
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "completed"
    );
    Ok(())
}

#[tokio::test]
async fn private_health_counts_admitted_uploads_until_rejected_or_cancelled()
-> Result<(), Box<dyn std::error::Error>> {
    async fn in_flight(router: axum::Router) -> Result<u64, Box<dyn std::error::Error>> {
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/private/v1/health")
                    .body(Body::empty())?,
            )
            .await?;
        let health: Value = serde_json::from_slice(&to_bytes(response.into_body(), MIB).await?)?;
        health["inFlight"]
            .as_u64()
            .ok_or_else(|| "missing inFlight".into())
    }

    let router = app_with_config(ServerConfig {
        max_concurrency: Some(1),
        ..Default::default()
    });
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    sender
        .send(Ok::<_, std::io::Error>(Bytes::from_static(b"\0\0")))
        .await?;
    let upload = tokio::spawn(
        router.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/private/v1/convert")
                .header(
                    CONTENT_TYPE,
                    "application/vnd.stencila.conversion.v1+frames",
                )
                .body(Body::from_stream(ReceiverStream::new(receiver)))?,
        ),
    );

    for _ in 0..20 {
        if in_flight(router.clone()).await? == 1 {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(in_flight(router.clone()).await?, 1);

    upload.abort();
    let _ = upload.await;
    drop(sender);
    assert_eq!(in_flight(router.clone()).await?, 0);

    let rejected = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/private/v1/convert")
                .header(
                    CONTENT_TYPE,
                    "application/vnd.stencila.conversion.v1+frames",
                )
                .body(Body::from("invalid"))?,
        )
        .await?;
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    assert_eq!(in_flight(router).await?, 0);
    Ok(())
}

#[tokio::test]
async fn accepts_the_javascript_conversion_v1_fixture() -> Result<(), Box<dyn std::error::Error>> {
    // Generated by the JavaScript frame encoder used by Cirrus. Keeping the bytes checked in makes
    // field casing, big-endian framing, and payload placement a cross-language compatibility test.
    let body = decode_hex_fixture(include_str!("fixtures/conversion-v1-request.hex"))?;
    let request = Request::builder()
        .method("POST")
        .uri("/private/v1/convert")
        .header(
            CONTENT_TYPE,
            "application/vnd.stencila.conversion.v1+frames",
        )
        .body(Body::from(body))?;
    let response = app().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), 2 * MIB).await?)?;
    assert_eq!(frames[0].0["jobId"], "fixture-job");
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "completed"
    );
    Ok(())
}

#[tokio::test]
async fn private_bad_jats_returns_a_terminal_failed_trailer()
-> Result<(), Box<dyn std::error::Error>> {
    let response = app()
        .oneshot(private_request(
            "jats",
            "markdown",
            "article.xml",
            None,
            &[("article.xml", b"<article><unclosed>")],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
    assert_eq!(frames.first().ok_or("missing result")?.0["kind"], "result");
    let trailer = &frames.last().ok_or("missing trailer")?.0;
    assert_eq!(trailer["status"], "failed");
    assert_eq!(trailer["error"]["code"], "conversion_failed");
    assert_eq!(trailer["modelCalls"], 0);
    Ok(())
}

#[tokio::test]
async fn privately_converts_jats_and_retains_original_media()
-> Result<(), Box<dyn std::error::Error>> {
    let jats = br#"<article><front><article-meta><title-group><article-title>Private test</article-title></title-group></article-meta></front><body><p>Text.</p><fig><graphic href="figure.png"/></fig></body></article>"#;
    let figure = b"unchanged-scientific-media";
    let response = app()
        .oneshot(private_request(
            "jats",
            "json",
            "article.xml",
            None,
            &[("article.xml", jats), ("figure.png", figure)],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), 4 * MIB).await?)?;
    assert_eq!(
        frames.first().and_then(|(frame, _)| frame["kind"].as_str()),
        Some("result")
    );
    assert!(
        frames
            .iter()
            .any(|(frame, _)| frame["kind"] == "artifact" && frame["role"] == "primary")
    );
    assert!(
        frames
            .iter()
            .any(|(frame, _)| frame["kind"] == "artifact" && frame["path"] == "figure.png")
    );
    assert!(
        frames
            .iter()
            .any(|(frame, payload)| frame["kind"] == "part" && payload == figure)
    );
    let trailer = frames
        .iter()
        .find(|(frame, _)| frame["kind"] == "trailer")
        .ok_or("missing trailer")?;
    assert_eq!(trailer.0["status"], "completed");
    assert_eq!(trailer.0["modelCalls"], 0);
    Ok(())
}

#[tokio::test]
async fn private_output_rebases_nested_source_media() -> Result<(), Box<dyn std::error::Error>> {
    let jats = br#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><fig><graphic xlink:href="figure.png"/></fig></body></article>"#;
    let response = app()
        .oneshot(private_request(
            "jats",
            "markdown",
            "content/article.xml",
            None,
            &[
                ("content/article.xml", jats),
                ("content/figure.png", b"figure"),
            ],
        )?)
        .await?;
    let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
    let markdown: Vec<u8> = frames
        .iter()
        .filter(|(frame, _)| frame["kind"] == "part" && frame["id"] == "primary")
        .flat_map(|(_, payload)| payload.iter().copied())
        .collect();
    assert!(String::from_utf8(markdown)?.contains("content/figure.png"));
    Ok(())
}

#[tokio::test]
async fn private_json_reports_missing_media() -> Result<(), Box<dyn std::error::Error>> {
    let json =
        br#"{"type":"Article","content":[{"type":"ImageObject","contentUrl":"missing.png"}]}"#;
    let response = app()
        .oneshot(private_request(
            "json",
            "markdown",
            "article.json",
            None,
            &[("article.json", json)],
        )?)
        .await?;
    let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
    let trailer = &frames.last().ok_or("missing trailer")?.0;
    assert_eq!(trailer["status"], "completed");
    assert_eq!(trailer["missingAssets"][0]["path"], "missing.png");
    Ok(())
}

#[tokio::test]
async fn private_large_diagnostic_report_has_a_terminal_trailer()
-> Result<(), Box<dyn std::error::Error>> {
    let content = (0..300)
        .map(|index| {
            serde_json::json!({
                "type": "ImageObject",
                "contentUrl": format!("{index:03}-{}.png", "x".repeat(950))
            })
        })
        .collect::<Vec<_>>();
    let json = serde_json::to_vec(&serde_json::json!({"type": "Article", "content": content}))?;
    let response = app()
        .oneshot(private_request(
            "json",
            "markdown",
            "article.json",
            None,
            &[("article.json", &json)],
        )?)
        .await?;
    let frames = private_response_frames(&to_bytes(response.into_body(), 2 * MIB).await?)?;
    let trailer = &frames.last().ok_or("missing trailer")?.0;
    assert_eq!(trailer["kind"], "trailer");
    assert_eq!(trailer["status"], "completed");
    assert_eq!(trailer["missingAssetsTruncated"], true);
    Ok(())
}

#[tokio::test]
async fn privately_converts_a_real_pmc_jats_fixture() -> Result<(), Box<dyn std::error::Error>> {
    // This is a checked-in article retrieved from PMC, rather than a mocked provider response.
    // Provider retrieval itself remains outside ordinary converter CI.
    let jats = include_bytes!("../../codec-jats/tests/examples/pmc-12431436.jats.xml");
    let response = app()
        .oneshot(private_request(
            "jats",
            "json",
            "pmc-12431436.xml",
            None,
            &[("pmc-12431436.xml", jats)],
        )?)
        .await?;
    let status = response.status();
    let body = to_bytes(response.into_body(), 20 * MIB).await?;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let frames = private_response_frames(&body)?;
    let primary_id = frames
        .iter()
        .find(|(frame, _)| frame["kind"] == "artifact" && frame["role"] == "primary")
        .and_then(|(frame, _)| frame["id"].as_str())
        .ok_or("missing primary artifact")?;
    let primary: Vec<u8> = frames
        .iter()
        .filter(|(frame, _)| frame["kind"] == "part" && frame["id"] == primary_id)
        .flat_map(|(_, payload)| payload.iter().copied())
        .collect();
    let document: Value = serde_json::from_slice(&primary)?;
    assert_eq!(document["type"], "Article");
    let trailer = frames
        .iter()
        .find(|(frame, _)| frame["kind"] == "trailer")
        .ok_or("missing trailer")?;
    assert_eq!(trailer.0["modelCalls"], 0);
    Ok(())
}

#[tokio::test]
async fn private_bundle_rejects_traversal_and_hash_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let traversal = app()
        .oneshot(private_request(
            "jats",
            "markdown",
            "../article.xml",
            None,
            &[("../article.xml", b"<article/>")],
        )?)
        .await?;
    assert_eq!(traversal.status(), StatusCode::BAD_REQUEST);

    let mut request = private_request(
        "jats",
        "markdown",
        "article.xml",
        None,
        &[("article.xml", b"<article/>")],
    )?;
    let body = std::mem::replace(request.body_mut(), Body::empty());
    let mut bytes = to_bytes(body, MIB).await?.to_vec();
    if let Some(last) = bytes.iter_mut().rfind(|byte| **byte == b'>') {
        *last = b'?'
    }
    *request.body_mut() = Body::from(bytes);
    let mismatch = app().oneshot(request).await?;
    assert_eq!(mismatch.status(), StatusCode::BAD_REQUEST);

    let collision = app()
        .oneshot(private_request(
            "jats",
            "markdown",
            "article.xml",
            None,
            &[
                (
                    "article.xml",
                    b"<article><body><p>text</p></body></article>",
                ),
                ("document.md", b"caller-controlled collision"),
            ],
        )?)
        .await?;
    assert_eq!(collision.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(collision.into_body(), MIB).await?)?;
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "failed"
    );
    Ok(())
}

#[tokio::test]
async fn private_conversion_rejects_unconfined_document_references()
-> Result<(), Box<dyn std::error::Error>> {
    for reference in [
        "/etc/passwd",
        "../secret.png",
        "file:///etc/passwd",
        "https://example.com/figure.png",
        "data:image/svg+xml,<svg><script>alert(1)</script></svg>",
    ] {
        let json = format!(r#"{{"type":"ImageObject","contentUrl":"{reference}"}}"#);
        let response = app()
            .oneshot(private_request(
                "json",
                "markdown",
                "article.json",
                None,
                &[("article.json", json.as_bytes())],
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
        assert_eq!(
            frames.last().ok_or("missing trailer")?.0["status"],
            "failed",
            "accepted {reference}"
        );
    }

    let json = br#"{"type":"Article","content":[{"type":"ImageObject","contentUrl":"file:///etc/passwd"}]}"#;
    let response = app()
        .oneshot(private_request(
            "json",
            "markdown",
            "article.json",
            None,
            &[("article.json", json)],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "failed"
    );

    let entity = String::from_utf8(
        include_bytes!("../../codec-jats/tests/examples/pmc-12431436.jats.xml").to_vec(),
    )?
    .replacen("pone.0332245.g001.jpg", "&#47;etc&#47;passwd", 1);
    let response = app()
        .oneshot(private_request(
            "jats",
            "markdown",
            "article.xml",
            None,
            &[("article.xml", entity.as_bytes())],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "failed"
    );

    let article_url = br#"{"type":"Article","url":"https://doi.org/10.1/example","content":[]}"#;
    let response = app()
        .oneshot(private_request(
            "json",
            "markdown",
            "article.json",
            None,
            &[("article.json", article_url)],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "completed"
    );
    Ok(())
}

fn synthetic_meca(
    manifest_href: &str,
    entries: &[(&str, &[u8])],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    archive.start_file("manifest.xml", options)?;
    write!(
        archive,
        r#"<manifest xmlns="http://manuscriptexchange.org"><item type="article"><instance media-type="application/xml" href="{manifest_href}"/></item>"#
    )?;
    for (path, _) in entries.iter().filter(|(path, _)| *path != manifest_href) {
        let kind = if path.ends_with(".jpg") {
            "figure"
        } else {
            "supplementary-material"
        };
        write!(
            archive,
            r#"<item type="{kind}"><instance href="{path}"/></item>"#
        )?;
    }
    write!(archive, "</manifest>")?;
    for (path, bytes) in entries {
        archive.start_file(path, options)?;
        archive.write_all(bytes)?;
    }
    Ok(archive.finish()?.into_inner())
}

#[tokio::test]
async fn private_meca_requires_the_manifest_selected_manuscript_and_retains_supplements()
-> Result<(), Box<dyn std::error::Error>> {
    let archive = synthetic_meca(
        "content/article.xml",
        &[
            (
                "content/article.xml",
                b"<article><body><p>MECA</p></body></article>",
            ),
            ("content/supplement.csv", b"a,b\n1,2\n"),
        ],
    )?;
    let response = app()
        .oneshot(private_request(
            "meca",
            "markdown",
            "article.meca",
            Some("content/article.xml"),
            &[("article.meca", &archive)],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), 4 * MIB).await?)?;
    let trailer = frames
        .iter()
        .find(|(frame, _)| frame["kind"] == "trailer")
        .ok_or("missing trailer")?;
    assert_eq!(
        trailer.0["supplements"][0]["path"],
        "content/supplement.csv"
    );
    assert!(
        frames
            .iter()
            .any(|(frame, payload)| frame["kind"] == "part" && payload == b"a,b\n1,2\n")
    );

    let inferred = app()
        .oneshot(private_request(
            "meca",
            "markdown",
            "article.meca",
            None,
            &[("article.meca", &archive)],
        )?)
        .await?;
    assert_eq!(inferred.status(), StatusCode::OK);

    let mismatch = app()
        .oneshot(private_request(
            "meca",
            "markdown",
            "article.meca",
            Some("content/other.xml"),
            &[("article.meca", &archive)],
        )?)
        .await?;
    assert_eq!(mismatch.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(mismatch.into_body(), MIB).await?)?;
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "failed"
    );
    Ok(())
}

#[tokio::test]
async fn private_converts_real_meca_with_a_doctype() -> Result<(), Box<dyn std::error::Error>> {
    let archive =
        include_bytes!("../../codec-meca/tests/examples/f2af58f0-7111-1014-b4ee-f306f628684d.meca");
    let response = app()
        .oneshot(private_request(
            "meca",
            "markdown",
            "article.meca",
            None,
            &[("article.meca", archive)],
        )?)
        .await?;
    let frames = private_response_frames(&to_bytes(response.into_body(), 20 * MIB).await?)?;
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "completed"
    );
    Ok(())
}

#[tokio::test]
async fn private_meca_rejects_archive_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let archive = synthetic_meca(
        "content/article.xml",
        &[
            ("content/article.xml", b"<article/>"),
            ("../escape.txt", b"escape"),
        ],
    )?;
    let response = app()
        .oneshot(private_request(
            "meca",
            "markdown",
            "article.meca",
            Some("content/article.xml"),
            &[("article.meca", &archive)],
        )?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
    assert_eq!(
        frames.last().ok_or("missing trailer")?.0["status"],
        "failed"
    );

    for entries in [vec![
        ("content", b"not-a-directory".as_slice()),
        ("content/article.xml", b"<article/>".as_slice()),
    ]] {
        let archive = synthetic_meca("content/article.xml", &entries)?;
        let response = app()
            .oneshot(private_request(
                "meca",
                "markdown",
                "article.meca",
                None,
                &[("article.meca", &archive)],
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let frames = private_response_frames(&to_bytes(response.into_body(), MIB).await?)?;
        assert_eq!(
            frames.last().ok_or("missing trailer")?.0["status"],
            "failed"
        );
    }
    Ok(())
}
