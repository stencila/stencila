use async_lsp::{
    ClientSocket, LanguageClient, ResponseError,
    lsp_types::{ExecuteCommandParams, MessageType, ShowMessageParams, Url},
};
use eyre::Result;
use serde_json::{Value, json};

use stencila_codecs::{DecodeOptions, EncodeOptions};
use stencila_document::Document;

use super::{internal_error, invalid_request, path_buf_arg, progress::create_progress};

/// Handle the pull-doc command
///
/// This pulls a document from a remote service
pub(crate) async fn pull_doc(
    params: ExecuteCommandParams,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let mut args = params.arguments.into_iter();

    let path = path_buf_arg(args.next())?;
    let options: Value = args.next().unwrap_or(json!({}));

    let target = options.get("target").and_then(|v| v.as_str());
    let merge = options
        .get("merge")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Create progress indicator
    let progress = create_progress(client.clone(), "Pulling document".to_string(), false).await;

    // Show initial source message
    if let Some(target_str) = target {
        let source_msg = match target_str {
            "gdoc" | "gdocs" => "from Google Docs".to_string(),
            "m365" => "from Microsoft 365 document".to_string(),
            url if url.starts_with("http") => format!("from {url}"),
            _ => format!("from {target_str}"),
        };
        progress.send((10, Some(source_msg))).ok();
    } else {
        progress
            .send((10, Some("checking tracked remotes".to_string())))
            .ok();
    }

    // Open the document
    progress
        .send((20, Some("opening".to_string())))
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

    // Determine service and URL
    progress
        .send((30, Some("determining remote".to_string())))
        .ok();
    use stencila_codecs::remotes::RemoteService;

    let (service, url) = if let Some(target_str) = target {
        match target_str {
            "gdoc" | "gdocs" => {
                let remotes = doc.remotes().await.map_err(internal_error)?;
                let url = remotes
                    .iter()
                    .find(|u| RemoteService::GoogleDocs.matches_url(u))
                    .ok_or_else(|| invalid_request("No Google Docs remote found"))?
                    .clone();
                (RemoteService::GoogleDocs, url)
            }
            "m365" => {
                let remotes = doc.remotes().await.map_err(internal_error)?;
                let url = remotes
                    .iter()
                    .find(|u| RemoteService::Microsoft365.matches_url(u))
                    .ok_or_else(|| invalid_request("No Microsoft 365 remote found"))?
                    .clone();
                (RemoteService::Microsoft365, url)
            }
            _ => {
                let url = Url::parse(target_str)
                    .map_err(|_| invalid_request(format!("Invalid target: {}", target_str)))?;
                let service = RemoteService::from_url(&url).ok_or_else(|| {
                    invalid_request(format!(
                        "URL {} is not from a supported remote service",
                        url
                    ))
                })?;
                (service, url)
            }
        }
    } else {
        // Use tracked remotes
        let remotes = doc.remotes().await.map_err(internal_error)?;
        if remotes.is_empty() {
            // Close progress before returning status response
            progress.send((100, None)).ok();
            // Return a status response indicating no remotes (not an error, requires user input)
            return Ok(Some(json!({
                "status": "no_remotes",
                "message": "No tracked remotes found. Please select a target."
            })));
        }

        // If multiple remotes, we need the user to specify which one
        if remotes.len() > 1 {
            // Close progress before returning status response
            progress.send((100, None)).ok();
            // Return status response with list of remotes for VSCode to prompt user
            let remotes_json: Vec<_> = remotes
                .iter()
                .filter_map(|url| {
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

        let remote_url = &remotes[0];
        let service = RemoteService::from_url(remote_url).ok_or_else(|| {
            invalid_request(format!(
                "Tracked remote {} is not from a supported service",
                remote_url
            ))
        })?;

        (service, remote_url.clone())
    };

    // Pull from remote
    progress
        .send((50, Some("fetching".to_string())))
        .ok();
    let modified_files = match stencila_codecs::pull(
        &service,
        &url,
        &path,
        merge,
        DecodeOptions::default(),
        EncodeOptions::default(),
    )
    .await
    {
        Ok(modified) => modified,
        Err(error) => {
            progress.send((100, None)).ok();
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Failed to pull document: {error}"),
                })
                .ok();
            return Ok(None);
        }
    };

    // Track the remote pull
    progress
        .send((80, Some("recording".to_string())))
        .ok();
    if let Err(error) = doc.track_remote_pulled(url).await {
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("Failed to track remote: {error}"),
            })
            .ok();
    }

    // Complete progress
    progress.send((100, None)).ok();

    Ok(Some(json!({ "modified_files": modified_files })))
}
