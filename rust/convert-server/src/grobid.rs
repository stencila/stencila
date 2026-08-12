//! GROBID-shaped compatibility routes
//!
//! These routes preserve the request shape used by GROBID and ScienceBeam
//! clients while returning JATS XML, the structured article XML format that
//! Stencila can encode. Route registration is the module's interface; request
//! parsing, content negotiation, and response policy stay behind that seam.

use std::path::Path;

use axum::{
    Router,
    body::Body,
    extract::{FromRequest, Multipart, State},
    http::{HeaderMap, Request, StatusCode, header::ACCEPT, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use stencila_codecs::Format;

use super::{
    AppError, AppState, ConvertRequest, Disposition, Mode, ServerConfig, execute_conversion,
    next_multipart_field, read_uploaded_file,
};

/// The media type returned for JATS XML
const JATS_RESPONSE_MEDIA_TYPE: &str = "application/xml; charset=utf-8";

/// Add the GROBID-shaped routes to the application
pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/isalive", get(isalive)).route(
        "/api/processFulltextDocument",
        post(process_fulltext_document).put(process_fulltext_document),
    )
}

/// GROBID-compatible liveness probe
async fn isalive() -> Response {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], "true").into_response()
}

/// Convert an uploaded document to JATS using GROBID's fulltext route shape
async fn process_fulltext_document(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Response, AppError> {
    if !accepts_jats(request.headers()) {
        return Err(AppError::new(
            StatusCode::NOT_ACCEPTABLE,
            "not_acceptable",
            concat!(
                "This endpoint returns JATS XML; accept `application/vnd.jats+xml`, ",
                "`application/xml`, `text/xml`, or `*/*`"
            ),
        ));
    }

    if !super::content_type_of(&request).starts_with("multipart/form-data") {
        return Err(AppError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Use multipart/form-data with the document in the `input` or `file` part",
        ));
    }

    let multipart = Multipart::from_request(request, &())
        .await
        .map_err(|error| {
            super::extractor_error(error.status(), error.body_text(), &state.config)
        })?;
    let request = request_from_multipart(multipart, &state.config).await?;
    let output = execute_conversion(&state, request).await?;

    // Encoding JATS to a string never produces sidecar files. Keep this guard
    // so a future encoder change cannot cause a ZIP to be labelled as XML.
    if output.disposition != Disposition::Inline {
        return Err(AppError::internal("JATS conversion was not inline"));
    }

    output.into_response(Some(JATS_RESPONSE_MEDIA_TYPE))
}

/// Parse a GROBID/ScienceBeam multipart request into a conversion request
async fn request_from_multipart(
    mut multipart: Multipart,
    config: &ServerConfig,
) -> Result<ConvertRequest, AppError> {
    let mut input = None;
    let mut file_count = 0usize;

    while let Some(field) = next_multipart_field(&mut multipart, config).await? {
        let is_file = field.file_name().is_some();
        if !is_file {
            // GROBID's optional control fields are accepted and ignored. A
            // filename-less field called `input` is not an uploaded document.
            continue;
        }

        file_count += 1;
        if file_count > 1 {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "multiple_inputs",
                "Provide exactly one document in the `input` or `file` part",
            ));
        }

        // GROBID uses `input`; ScienceBeam also accepts the legacy `file`
        // alias. Other file field names must not become inputs accidentally.
        let is_input = matches!(field.name(), Some("input" | "file"));
        if is_input {
            input = Some(read_uploaded_file(field, config).await?);
        }
    }

    let Some(input) = input else {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "missing_input",
            "Provide the document to convert in the `input` or `file` part",
        ));
    };

    let from = input_format(input.filename.as_deref());
    let source = input.into_source(Some(&from), "input.pdf").await?;

    Ok(ConvertRequest {
        source,
        to: Format::Jats,
        from: Some(from),
        mode: Some(Mode::Inline),
        compact: None,
        standalone: Some(true),
        embed_media: None,
    })
}

/// Infer the uploaded format, falling back to PDF like GROBID
fn input_format(filename: Option<&str>) -> Format {
    let format = filename
        .map(|filename| Format::from_path(Path::new(filename)))
        .unwrap_or(Format::Unknown);

    if format.is_unknown() || format.is_other() {
        Format::Pdf
    } else {
        format
    }
}

/// Whether the client accepts the JATS representation returned by this route
fn accepts_jats(headers: &HeaderMap) -> bool {
    let values = headers.get_all(ACCEPT);
    if values.iter().next().is_none() {
        return true;
    }

    let mut exact_range_seen = false;
    let mut exact_range_accepted = false;
    let mut wildcard_accepted = false;

    for value in values.iter() {
        let Ok(value) = value.to_str() else {
            continue;
        };
        if value.trim().is_empty() {
            return true;
        }

        for range in value.split(',').map(jats_media_range) {
            match range {
                JatsMediaRange::Exact { accepted } => {
                    exact_range_seen = true;
                    exact_range_accepted |= accepted;
                }
                JatsMediaRange::Wildcard { accepted } => wildcard_accepted |= accepted,
                JatsMediaRange::Other => {}
            }
        }
    }

    if exact_range_seen {
        exact_range_accepted
    } else {
        wildcard_accepted
    }
}

/// How one Accept header entry applies to the JATS response
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JatsMediaRange {
    Exact { accepted: bool },
    Wildcard { accepted: bool },
    Other,
}

fn jats_media_range(entry: &str) -> JatsMediaRange {
    let mut parts = entry.split(';');
    let media_type = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
    let mut quality = 1.0;

    for parameter in parts {
        let Some((name, value)) = parameter.split_once('=') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("q") {
            quality = value
                .trim()
                .parse::<f32>()
                .ok()
                .filter(|quality| *quality >= 0.0 && *quality <= 1.0)
                .unwrap_or_default();
            break;
        }
    }

    let accepted = quality > 0.0;
    match media_type.as_str() {
        "application/xml" | "text/xml" | "application/vnd.jats+xml" => {
            JatsMediaRange::Exact { accepted }
        }
        "*/*" => JatsMediaRange::Wildcard { accepted },
        _ => JatsMediaRange::Other,
    }
}
