use async_lsp::{
    ClientSocket, LanguageClient, ResponseError,
    lsp_types::{ExecuteCommandParams, MessageType, ShowMessageParams},
};
use chrono::Utc;
use eyre::Result;
use serde_json::{Value, json};

use stencila_dirs::closest_workspace_dir;
use stencila_document::Document;
use stencila_remotes::{get_all_remote_entries, update_remote_timestamp};

use super::internal_error;

/// Handle the push-docs command
///
/// This pushes all tracked documents to their remotes
pub(crate) async fn push_docs(
    params: ExecuteCommandParams,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let mut args = params.arguments.into_iter();
    let options: Value = args.next().unwrap_or(json!({}));

    let no_execute = options
        .get("no_execute")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let args_str = options.get("args").and_then(|v| v.as_str());

    // Get current directory and workspace
    let cwd = std::env::current_dir().map_err(internal_error)?;
    let workspace_dir = closest_workspace_dir(&cwd, false)
        .await
        .map_err(internal_error)?;

    // Get all files with remotes
    let file_entries = get_all_remote_entries(&workspace_dir)
        .await
        .map_err(internal_error)?;

    let Some(entries) = file_entries else {
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: "No files with remotes found".to_string(),
            })
            .ok();
        return Ok(None);
    };

    if entries.is_empty() {
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: "No files with remotes found".to_string(),
            })
            .ok();
        return Ok(None);
    }

    let mut total_successes = 0;
    let mut total_errors = 0;

    use stencila_remotes::RemoteService;

    for (file_path, remotes) in entries {
        // Open document
        let doc = match Document::open(&file_path, None).await {
            Ok(d) => d,
            Err(_) => {
                total_errors += remotes.len();
                continue;
            }
        };

        // Execute if needed
        if !no_execute {
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

            if doc
                .call(&arguments, stencila_node_execute::ExecuteOptions::default())
                .await
                .is_err()
            {
                total_errors += remotes.len();
                continue;
            }
        }

        // Push to each remote
        for remote_url in remotes.keys() {
            let Some(service) = RemoteService::from_url(remote_url) else {
                total_errors += 1;
                continue;
            };

            match stencila_codecs::push(
                &service,
                &doc.root().await,
                doc.path(),
                doc.file_name(),
                Some(remote_url),
                doc.path(),
                None, // LSP doesn't support dry-run yet
            )
            .await
            {
                Ok(result) => {
                    if let Some(doc_path) = doc.path() {
                        if update_remote_timestamp(
                            doc_path,
                            result.url().as_ref(),
                            None,
                            Some(Utc::now().timestamp() as u64),
                        )
                        .await
                        .is_ok()
                        {
                            total_successes += 1;
                        } else {
                            total_errors += 1;
                        }
                    } else {
                        total_successes += 1; // Count as success even if we can't track
                    }
                }
                Err(_) => {
                    total_errors += 1;
                }
            }
        }
    }

    Ok(Some(json!({
        "total": total_successes + total_errors,
        "succeeded": total_successes,
        "failed": total_errors,
    })))
}
