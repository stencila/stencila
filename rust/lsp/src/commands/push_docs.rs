use async_lsp::{
    ClientSocket, LanguageClient, ResponseError,
    lsp_types::{ExecuteCommandParams, MessageType, ShowMessageParams},
};
use eyre::Result;
use serde_json::{Value, json};

use stencila_document::Document;

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

    // Get current directory and all tracked files
    let cwd = std::env::current_dir().map_err(internal_error)?;
    let tracking_entries = Document::tracking_all(&cwd).await.map_err(internal_error)?;

    let Some(entries) = tracking_entries else {
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: "No tracked files found".to_string(),
            })
            .ok();
        return Ok(None);
    };

    // Filter to files with remotes
    let files_with_remotes: Vec<_> = entries
        .iter()
        .filter(|(_, tracking)| {
            tracking
                .remotes
                .as_ref()
                .is_some_and(|remotes| !remotes.is_empty())
        })
        .collect();

    if files_with_remotes.is_empty() {
        client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: "No tracked files with remotes found".to_string(),
            })
            .ok();
        return Ok(None);
    }

    let mut total_successes = 0;
    let mut total_errors = 0;

    use stencila_codecs::remotes::RemoteService;

    for (file_path, tracking) in files_with_remotes {
        let remotes = tracking
            .remotes
            .as_ref()
            .expect("tracking should have remotes");

        // Open document
        let doc = match Document::open(file_path, None).await {
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
            )
            .await
            {
                Ok(url) => {
                    if doc.track_remote_pushed(url).await.is_ok() {
                        total_successes += 1;
                    } else {
                        total_errors += 1;
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
