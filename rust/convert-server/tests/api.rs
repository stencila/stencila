use axum::{
    body::{Body, to_bytes},
    http::{
        Request, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
};
use serde_json::Value;
use std::io::{Cursor, Read};
use stencila_convert_server::app;
use tower::ServiceExt;
use zip::ZipArchive;

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

    let bytes = to_bytes(response.into_body(), 1024 * 1024).await?;
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

    let bytes = to_bytes(response.into_body(), 1024 * 1024).await?;
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

    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024).await?;
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

    let bytes = to_bytes(response.into_body(), 1024 * 1024).await?;
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

    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024).await?;
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

    let bytes = to_bytes(response.into_body(), 1024 * 1024).await?;
    let json: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(json["error"]["code"], "unsupported_url");

    Ok(())
}
