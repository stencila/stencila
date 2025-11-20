use std::collections::BTreeMap;

use async_lsp::{
    ClientSocket, LanguageClient, ResponseError,
    lsp_types::{MessageType, ShowMessageParams},
};
use eyre::Result;
use reqwest::Url;
use serde_json::{Value, json};

use stencila_cloud::delete_watch;
use stencila_remotes::RemoteService;

use super::{internal_error, invalid_request, path_buf_arg, progress::create_progress};

/// Handle the unwatch-doc command
///
/// This disables automatic sync for a document
pub(crate) async fn unwatch_doc(
    params: async_lsp::lsp_types::ExecuteCommandParams,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let mut args = params.arguments.into_iter();

    let path = path_buf_arg(args.next())?;
    let options: Value = args.next().unwrap_or(json!({}));

    // Extract options from JSON
    let target = options.get("target").and_then(|v| v.as_str());

    // Create progress indicator
    let progress = create_progress(client.clone(), "Removing watch".to_string(), false).await;

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

    // Get tracking information
    progress
        .send((30, Some("checking remotes".to_string())))
        .ok();
    let not_watched = || {
        progress.send((100, None)).ok();
        Ok(Some(json!({
            "status": "not_watched",
            "message": "Document is not being watched."
        })))
    };
    // Get remotes for this document
    let workspace_dir = stencila_dirs::closest_workspace_dir(&path, false)
        .await
        .map_err(internal_error)?;
    let remote_infos = stencila_remotes::get_remotes_for_path(&path, Some(&workspace_dir))
        .await
        .map_err(internal_error)?;
    if remote_infos.is_empty() {
        return not_watched();
    }

    // Convert to BTreeMap
    let remotes: BTreeMap<Url, stencila_remotes::RemoteInfo> = remote_infos
        .into_iter()
        .map(|r| (r.url.clone(), r))
        .collect();

    // Determine which remote to unwatch based on target argument
    let (remote_url, remote_info) = if let Some(target_str) = target {
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
        // No target specified - check if there's only one watched remote
        let watched_remotes: Vec<_> = remotes
            .iter()
            .filter(|(_, info)| info.is_watched())
            .collect();

        if watched_remotes.is_empty() {
            return not_watched();
        }

        if watched_remotes.len() > 1 {
            progress.send((100, None)).ok();
            let remotes_json: Vec<_> = watched_remotes
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
                "status": "multiple_watched",
                "message": "Multiple watched remotes found. Please select one.",
                "remotes": remotes_json
            })));
        }

        // Get the single watched remote
        let (url, info) = watched_remotes[0];
        (url.clone(), info.clone())
    };

    // Check if remote is actually being watched
    if !remote_info.is_watched() {
        progress.send((100, None)).ok();
        return Ok(Some(json!({
            "status": "not_watched",
            "message": format!("Remote {remote_url} is not being watched.")
        })));
    }

    // Call Cloud API to delete watch
    progress.send((50, Some("deleting watch".to_string()))).ok();
    let watch_id = remote_info
        .watch_id
        .as_ref()
        .ok_or_else(|| internal_error("No watch ID found"))?;

    if let Err(error) = delete_watch(watch_id).await {
        progress.send((100, None)).ok();
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("Failed to delete watch: {error}"),
            })
            .ok();
        return Ok(None);
    }

    // Clear watch metadata (but preserve other tracking info)
    progress
        .send((80, Some("updating config".to_string())))
        .ok();

    // Remove watch ID from stencila.yaml config
    if let Err(error) = stencila_remotes::update_watch_id(&path, remote_url.as_ref(), None).await {
        progress.send((100, None)).ok();
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("Failed to update config: {error}"),
            })
            .ok();
        return Ok(None);
    }

    // Complete progress and send success response
    progress.send((100, None)).ok();
    Ok(Some(json!({
        "status": "success",
        "message": "Watch removed successfully."
    })))
}
