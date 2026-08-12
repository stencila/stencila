use axum::{
    body::{Body, to_bytes},
    http::{
        Method, Request, StatusCode,
        header::{ACCEPT, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE},
    },
};
use serde_json::Value;
use std::io::{Cursor, Read};
use stencila_convert_server::{ServerConfig, app, app_with_config};
use tower::ServiceExt;
use zip::ZipArchive;

const MIB: usize = 1024 * 1024;

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
