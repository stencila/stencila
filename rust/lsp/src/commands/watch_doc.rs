use async_lsp::{
    ClientSocket, LanguageClient, ResponseError,
    lsp_types::{MessageType, ShowMessageParams},
};
use eyre::Result;
use serde_json::{Value, json};

use stencila_cloud::{WatchRequest, create_watch};
use stencila_codec_utils::{git_info, validate_file_on_default_branch};
use stencila_codecs::remotes::RemoteService;
use stencila_document::Document;

use super::{internal_error, invalid_request, path_buf_arg, progress::create_progress};

/// Handle the watch-doc command
///
/// This enables automatic sync between a document and its remote service
pub(crate) async fn watch_doc(
    params: async_lsp::lsp_types::ExecuteCommandParams,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let mut args = params.arguments.into_iter();

    let path = path_buf_arg(args.next())?;
    let options: Value = args.next().unwrap_or(json!({}));

    // Extract options from JSON
    let target = options.get("target").and_then(|v| v.as_str());
    let direction = options.get("direction").and_then(|v| v.as_str());

    // Create progress indicator
    let progress = create_progress(client.clone(), "Setting up watch".to_string(), false).await;

    // Validate file exists
    if !path.exists() {
        progress.send((100, None)).ok();
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("File `{}` does not exist", path.display()),
            })
            .ok();
        return Ok(None);
    }

    // Validate file is on default branch
    progress
        .send((10, Some("validating git status".to_string())))
        .ok();
    if let Err(error) = validate_file_on_default_branch(&path) {
        progress.send((100, None)).ok();
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("Cannot enable watch: {error}"),
            })
            .ok();
        return Ok(None);
    }

    // Get git repository information
    let git_info = match git_info(&path) {
        Ok(info) => info,
        Err(error) => {
            progress.send((100, None)).ok();
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Cannot enable watch: {error}"),
                })
                .ok();
            return Ok(None);
        }
    };

    let Some(repo_url) = git_info.origin else {
        progress.send((100, None)).ok();
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: "Cannot enable watch: No git remote origin found".to_string(),
            })
            .ok();
        return Ok(None);
    };

    // Open the document
    progress
        .send((20, Some("opening document".to_string())))
        .ok();
    let doc = match Document::open(&path, None).await {
        Ok(doc) => doc,
        Err(error) => {
            progress.send((100, None)).ok();
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Failed to open document: {error}"),
                })
                .ok();
            return Ok(None);
        }
    };

    // Get tracking information
    progress
        .send((30, Some("checking remotes".to_string())))
        .ok();
    let no_remotes = || {
        progress.send((100, None)).ok();
        Ok(Some(json!({
            "status": "no_remotes",
            "message": "No tracked remotes found. Please push to a remote first."
        })))
    };
    let Some((.., Some(tracking))) = doc.tracking().await.map_err(internal_error)? else {
        return no_remotes();
    };
    let Some(remotes) = tracking.remotes else {
        return no_remotes();
    };
    if remotes.is_empty() {
        return no_remotes();
    }

    // Determine which remote to watch based on target argument or tracked remotes
    let (remote_url, mut remote_info) = if let Some(target_str) = target {
        // Parse target as service shorthand or URL
        let target_url = match target_str {
            "gdoc" | "gdocs" => {
                // Find the Google Docs remote
                remotes
                    .iter()
                    .find(|(url, _)| RemoteService::GoogleDocs.matches_url(url))
                    .ok_or_else(|| {
                        invalid_request("No Google Docs remote found for this document")
                    })?
                    .0
                    .clone()
            }
            "m365" => {
                // Find the M365 remote
                remotes
                    .iter()
                    .find(|(url, _)| RemoteService::Microsoft365.matches_url(url))
                    .ok_or_else(|| {
                        invalid_request("No Microsoft 365 remote found for this document")
                    })?
                    .0
                    .clone()
            }
            _ => {
                // Try to parse as URL
                async_lsp::lsp_types::Url::parse(target_str).map_err(|_| {
                    invalid_request(format!(
                        "Invalid target or service: '{}'. Use 'gdoc', 'm365', or a full URL.",
                        target_str
                    ))
                })?
            }
        };

        // Find the remote in the tracked remotes
        remotes
            .into_iter()
            .find(|(url, _)| url == &target_url)
            .ok_or_else(|| invalid_request("Remote target not found in tracked remotes"))?
    } else {
        // No target specified - check if there's only one remote
        if remotes.len() > 1 {
            progress.send((100, None)).ok();
            let remotes_json: Vec<_> = remotes
                .iter()
                .filter_map(|(url, _)| {
                    RemoteService::from_url(url).map(|service| {
                        json!({
                            "service": service.display_name(),
                            "url": url.to_string()
                        })
                    })
                })
                .collect();

            return Ok(Some(json!({
                "status": "multiple_remotes",
                "message": "Multiple remotes found. Please select one.",
                "remotes": remotes_json
            })));
        }

        // Get the single remote
        remotes
            .into_iter()
            .next()
            .ok_or_else(|| internal_error("No remote found (this should not happen)"))?
    };

    // Check if already being watched
    if remote_info.is_watched() {
        progress.send((100, None)).ok();
        let service_name = RemoteService::from_url(&remote_url)
            .map(|s| s.display_name().to_string())
            .unwrap_or_else(|| remote_url.to_string());
        return Ok(Some(json!({
            "status": "already_watched",
            "message": format!("Document is already being watched on {service_name}")
        })));
    }

    // Get file path relative to repo root
    let file_path = git_info.path.unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Create watch request
    progress.send((50, Some("creating watch".to_string()))).ok();
    let request = WatchRequest {
        remote_url: remote_url.to_string(),
        repo_url,
        file_path,
        direction: direction.map(String::from),
        pr_mode: None,          // Use default
        debounce_seconds: None, // Use default
    };

    // Call Cloud API to create watch
    let response = match create_watch(request).await {
        Ok(response) => response,
        Err(error) => {
            progress.send((100, None)).ok();
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Failed to create watch: {error}"),
                })
                .ok();
            return Ok(None);
        }
    };

    // Update docs.json with watch metadata
    progress
        .send((80, Some("updating tracking".to_string())))
        .ok();
    remote_info.watch_id = Some(response.id.to_string());
    remote_info.watch_direction = direction.and_then(|d| d.parse().ok());

    if let Err(error) = doc.track(Some((remote_url.clone(), remote_info))).await {
        progress.send((100, None)).ok();
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("Failed to update watch tracking: {error}"),
            })
            .ok();
        return Ok(None);
    }

    // Complete progress
    progress.send((100, None)).ok();

    Ok(Some(json!({
        "status": "success",
        "watch_id": response.id.to_string(),
        "direction": direction.unwrap_or("bi-directional"),
        "remote_url": remote_url.to_string()
    })))
}
