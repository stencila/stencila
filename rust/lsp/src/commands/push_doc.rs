use async_lsp::{
    ClientSocket, LanguageClient, ResponseError,
    lsp_types::{ExecuteCommandParams, MessageType, ShowMessageParams, Url},
};
use chrono::Utc;
use eyre::Result;
use serde_json::{Value, json};

use stencila_document::Document;
use stencila_remotes::{
    RemoteService, get_remotes_for_path, update_remote_timestamp, update_watch_id,
};

use super::{internal_error, invalid_request, path_buf_arg, progress::create_progress};

/// Handle the push-doc command
///
/// This pushes a document to a remote service (Google Docs, Microsoft 365, etc.)
pub(crate) async fn push_doc(
    params: ExecuteCommandParams,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let mut args = params.arguments.into_iter();

    let path = path_buf_arg(args.next())?;
    let options: Value = args.next().unwrap_or(json!({}));

    // Extract options from JSON
    let target = options.get("target").and_then(|v| v.as_str());
    let force_new = options
        .get("force_new")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let no_execute = options
        .get("no_execute")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let watch = options
        .get("watch")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let push_all_remotes = options
        .get("push_all_remotes")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let args_str = options.get("args").and_then(|v| v.as_str());

    // Create progress indicator
    let progress = create_progress(client.clone(), "Pushing document".to_string(), false).await;

    // Show initial target message
    if let Some(target_str) = target {
        let target_msg = match target_str {
            "gdoc" | "gdocs" => "creating new Google Doc".to_string(),
            "m365" => "creating new Microsoft 365 document".to_string(),
            url if url.starts_with("http") => format!("to {url}"),
            _ => format!("to {target_str}"),
        };
        progress.send((10, Some(target_msg))).ok();
    } else {
        progress
            .send((10, Some("checking tracked remotes".to_string())))
            .ok();
    }

    // Open the document
    progress.send((20, Some("opening".to_string()))).ok();
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

    // Execute document if not skipped
    if !no_execute {
        progress.send((30, Some("executing".to_string()))).ok();
        let arguments: Vec<(&str, &str)> = args_str
            .unwrap_or("")
            .split_whitespace()
            .filter_map(|arg| {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((parts[0], parts[1]))
                } else {
                    None
                }
            })
            .collect();

        if let Err(error) = doc
            .call(&arguments, stencila_node_execute::ExecuteOptions::default())
            .await
        {
            progress.send((100, None)).ok();
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Failed to execute document: {error}"),
                })
                .ok();
            return Ok(None);
        }
    }

    // Determine service and URL
    progress
        .send((40, Some("determining remote".to_string())))
        .ok();

    let (service, existing_url) = if let Some(target_str) = target {
        // Parse target as service or URL
        match target_str {
            "gdoc" | "gdocs" => {
                let remote_infos = get_remotes_for_path(&path, None)
                    .await
                    .map_err(internal_error)?;
                let url = remote_infos
                    .iter()
                    .find(|info| RemoteService::GoogleDocs.matches_url(&info.url))
                    .map(|info| info.url.clone());
                (
                    RemoteService::GoogleDocs,
                    if force_new { None } else { url },
                )
            }
            "m365" => {
                let remote_infos = get_remotes_for_path(&path, None)
                    .await
                    .map_err(internal_error)?;
                let url = remote_infos
                    .iter()
                    .find(|info| RemoteService::Microsoft365.matches_url(&info.url))
                    .map(|info| info.url.clone());
                (
                    RemoteService::Microsoft365,
                    if force_new { None } else { url },
                )
            }
            _ => {
                // Try to parse as URL
                let url = Url::parse(target_str)
                    .map_err(|_| invalid_request(format!("Invalid target: {target_str}")))?;
                let service = RemoteService::from_url(&url).ok_or_else(|| {
                    invalid_request(format!(
                        "URL {} is not from a supported remote service",
                        url
                    ))
                })?;
                (service, if force_new { None } else { Some(url) })
            }
        }
    } else {
        // Use configured remotes
        let remote_infos = get_remotes_for_path(&path, None)
            .await
            .map_err(internal_error)?;
        if remote_infos.is_empty() {
            // Close progress before returning status response
            progress.send((100, None)).ok();
            // Return a status response indicating no remotes (not an error, requires user input)
            return Ok(Some(json!({
                "status": "no_remotes",
                "message": "No configured remotes found. Please select a target."
            })));
        }

        // If multiple remotes, either push to all or return list for user selection
        if remote_infos.len() > 1 {
            if push_all_remotes {
                // Push to all remotes
                progress
                    .send((
                        40,
                        Some(format!("pushing to {} remotes", remote_infos.len())),
                    ))
                    .ok();

                let mut successes: Vec<String> = Vec::new();
                let mut failures: Vec<(String, String)> = Vec::new();

                for (index, remote_info) in remote_infos.iter().enumerate() {
                    let remote_service = match RemoteService::from_url(&remote_info.url) {
                        Some(svc) => svc,
                        None => {
                            failures.push((
                                remote_info.url.to_string(),
                                format!(
                                    "URL {} is not from a supported remote service",
                                    remote_info.url
                                ),
                            ));
                            continue;
                        }
                    };

                    // Update progress for each remote
                    let remote_progress = 40 + (40 * (index + 1) / remote_infos.len()) as u32;
                    progress
                        .send((
                            remote_progress,
                            Some(format!("pushing to {}", remote_service.display_name())),
                        ))
                        .ok();

                    match stencila_codecs::push(
                        &remote_service,
                        &doc.root().await,
                        doc.path(),
                        doc.file_name(),
                        Some(&remote_info.url),
                        doc.path(),
                        None, // LSP doesn't support dry-run yet
                    )
                    .await
                    {
                        Ok(result) => {
                            let url = result.url();
                            if let Some(doc_path) = doc.path() {
                                if let Err(error) = update_remote_timestamp(
                                    doc_path,
                                    url.as_ref(),
                                    None,
                                    Some(Utc::now().timestamp() as u64),
                                )
                                .await
                                {
                                    failures.push((
                                        remote_info.url.to_string(),
                                        format!("Failed to track remote: {}", error),
                                    ));
                                } else {
                                    successes.push(url.to_string());
                                }
                            } else {
                                successes.push(url.to_string());
                            }
                        }
                        Err(error) => {
                            failures.push((remote_info.url.to_string(), error.to_string()));
                        }
                    }
                }

                // Complete progress
                progress.send((100, None)).ok();

                // Show appropriate message based on results
                if !failures.is_empty() && successes.is_empty() {
                    client
                        .show_message(ShowMessageParams {
                            typ: MessageType::ERROR,
                            message: format!("Failed to push to all {} remotes", failures.len()),
                        })
                        .ok();
                } else if !failures.is_empty() {
                    client
                        .show_message(ShowMessageParams {
                            typ: MessageType::WARNING,
                            message: format!(
                                "Pushed to {} remote(s), {} failed",
                                successes.len(),
                                failures.len()
                            ),
                        })
                        .ok();
                } else {
                    client
                        .show_message(ShowMessageParams {
                            typ: MessageType::INFO,
                            message: format!("Successfully pushed to {} remotes", successes.len()),
                        })
                        .ok();
                }

                return Ok(Some(json!({
                    "status": "success_multiple",
                    "success_count": successes.len(),
                    "fail_count": failures.len(),
                    "successes": successes,
                    "failures": failures
                })));
            } else {
                // Close progress before returning status response
                progress.send((100, None)).ok();
                // Return status response with list of remotes for VSCode to prompt user
                let remotes_json: Vec<_> = remote_infos
                    .iter()
                    .filter_map(|info| {
                        RemoteService::from_url(&info.url).map(|service| {
                            json!({
                                "service": service.display_name(),
                                "url": info.url.to_string()
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
        }

        let remote_info = &remote_infos[0];
        let service = RemoteService::from_url(&remote_info.url).ok_or_else(|| {
            invalid_request(format!(
                "Configured remote {} is not from a supported service",
                remote_info.url
            ))
        })?;

        (
            service,
            if force_new {
                None
            } else {
                Some(remote_info.url.clone())
            },
        )
    };

    // Push to remote
    progress.send((60, Some("sending".to_string()))).ok();
    let url = match stencila_codecs::push(
        &service,
        &doc.root().await,
        doc.path(),
        doc.file_name(),
        existing_url.as_ref(),
        doc.path(),
        None, // LSP doesn't support dry-run yet
    )
    .await
    {
        Ok(result) => result.url(),
        Err(error) => {
            progress.send((100, None)).ok();
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Failed to push document: {error}"),
                })
                .ok();
            return Ok(None);
        }
    };

    // Track the remote
    progress.send((80, Some("recording".to_string()))).ok();
    if let Some(doc_path) = doc.path()
        && let Err(error) = update_remote_timestamp(
            doc_path,
            url.as_ref(),
            None,
            Some(Utc::now().timestamp() as u64),
        )
        .await
    {
        progress.send((100, None)).ok();
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("Failed to track remote: {error}"),
            })
            .ok();
        return Ok(None);
    }

    // Handle watch mode if requested
    if watch {
        use stencila_cloud::{WatchRequest, create_watch};
        use stencila_codec_utils::{git_file_info, validate_file_on_default_branch};

        // Validate file is on default branch
        if let Err(error) = validate_file_on_default_branch(&path) {
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Cannot enable watch: {error}"),
                })
                .ok();
            // Don't fail the whole push, just skip watch
        } else {
            // Get git info
            let git_file_info = match git_file_info(&path) {
                Ok(info) => info,
                Err(error) => {
                    client
                        .show_message(ShowMessageParams {
                            typ: MessageType::ERROR,
                            message: format!("Cannot enable watch: {error}"),
                        })
                        .ok();
                    // Don't fail the whole push
                    return Ok(Some(json!({ "url": url.to_string() })));
                }
            };

            // Ensure workspace exists to get workspace_id
            let workspace_id = match stencila_cloud::ensure_workspace(&path).await {
                Ok((id, _)) => id,
                Err(error) => {
                    client
                        .show_message(ShowMessageParams {
                            typ: MessageType::ERROR,
                            message: format!("Cannot enable watch: {error}"),
                        })
                        .ok();
                    return Ok(Some(json!({ "url": url.to_string() })));
                }
            };

            let file_path = git_file_info.path.unwrap_or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

            let direction = options.get("direction").and_then(|v| v.as_str());
            let pr_mode = options.get("pr_mode").and_then(|v| v.as_str());
            let debounce_seconds = options.get("debounce_seconds").and_then(|v| v.as_u64());

            let request = WatchRequest {
                remote_url: url.to_string(),
                file_path,
                direction: direction.map(String::from),
                pr_mode: pr_mode.map(String::from),
                debounce_seconds,
            };

            match create_watch(&workspace_id, request).await {
                Ok(response) => {
                    // Store watch ID in stencila.toml config
                    if let Err(error) =
                        update_watch_id(&path, url.as_ref(), Some(response.id.to_string())).await
                    {
                        client
                            .show_message(ShowMessageParams {
                                typ: MessageType::ERROR,
                                message: format!("Failed to update watch tracking: {error}"),
                            })
                            .ok();
                    }
                }
                Err(error) => {
                    client
                        .show_message(ShowMessageParams {
                            typ: MessageType::ERROR,
                            message: format!("Failed to create watch: {error}"),
                        })
                        .ok();
                }
            }
        }
    }

    // Complete progress
    progress.send((100, None)).ok();

    Ok(Some(json!({ "url": url.to_string() })))
}
