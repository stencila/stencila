//! Handling of custom requests and notifications related to document tracking

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use async_lsp::lsp_types::request::Request;
use serde::{Deserialize, Serialize};

use stencila_codec_utils::git_info;
use stencila_dirs::closest_workspace_dir;
use stencila_document::{Document, DocumentTrackingEntries, RemoteStatus};
use stencila_remotes::{RemoteService, WatchDirection, calculate_remote_statuses};
use stencila_schema::NodeId;

/// Enriched document tracking with service information for display
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedDocumentTracking {
    pub id: NodeId,
    pub cached_at: Option<u64>,
    pub added_at: Option<u64>,
    pub remotes: Option<BTreeMap<String, EnrichedDocumentRemote>>,
}

/// Enriched document remote with service display information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedDocumentRemote {
    pub pulled_at: Option<u64>,
    pub pushed_at: Option<u64>,
    pub watch_id: Option<String>,
    pub watch_direction: Option<WatchDirection>,

    // Enriched fields for display
    pub service_name: Option<String>,
    pub display_name: Option<String>,

    // Sync status (calculated by comparing with remote)
    pub status: Option<RemoteStatus>,

    // Watch status fields
    pub watch_status: Option<String>,
    pub watch_status_summary: Option<String>,
    pub watch_last_error: Option<String>,

    // Pull request information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_pr: Option<PullRequestInfo>,
}

/// Pull request information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestInfo {
    pub status: String,
    pub url: String,
}

pub type EnrichedDocumentTrackingEntries = BTreeMap<PathBuf, EnrichedDocumentTracking>;

/// List document tracking information with enriched service details
pub async fn list() -> EnrichedDocumentTrackingEntries {
    // Get the current working directory as the starting point
    let path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    // Get all tracked documents
    let entries = match Document::tracking_all(&path).await {
        Ok(Some(entries)) => entries,
        Ok(None) => DocumentTrackingEntries::default(),
        Err(error) => {
            tracing::error!("Failed to get document tracking: {error}");
            DocumentTrackingEntries::default()
        }
    };

    // Get workspace directory and repo URL for watch details
    let workspace_dir = closest_workspace_dir(&path, false).await.ok();
    let repo_url = workspace_dir
        .as_ref()
        .and_then(|dir| git_info(dir).ok())
        .and_then(|info| info.origin);

    // Fetch watch details from API
    let watch_details_map: HashMap<u64, stencila_cloud::WatchDetailsResponse> =
        match stencila_cloud::get_watches(repo_url.as_deref()).await {
            Ok(watches) => watches.into_iter().map(|w| (w.id, w)).collect(),
            Err(error) => {
                tracing::debug!("Failed to fetch watch details from API: {error}");
                HashMap::new()
            }
        };

    // Enrich with service information and calculate remote statuses
    let mut enriched_entries = BTreeMap::new();

    for (path, tracking) in entries {
        // Get document status and modified time
        let (doc_status, modified_at) = if let Some(workspace_dir) = &workspace_dir {
            tracking.status(workspace_dir, &path)
        } else {
            (RemoteStatus::Unknown, None)
        };

        // Get remotes for this path from the remotes crate
        let remotes_info = if let Some(workspace_dir) = &workspace_dir {
            match stencila_remotes::get_remotes_for_path(&path, Some(workspace_dir)).await {
                Ok(remotes) => {
                    let remote_map: BTreeMap<_, _> =
                        remotes.into_iter().map(|r| (r.url.clone(), r)).collect();
                    Some(remote_map)
                }
                Err(_) => None,
            }
        } else {
            None
        };

        // Fetch remote statuses (this compares with actual remote modification times)
        let remote_statuses = if let Some(ref remotes) = remotes_info {
            calculate_remote_statuses(remotes, doc_status, modified_at).await
        } else {
            BTreeMap::new()
        };

        let enriched_remotes = remotes_info.map(|remotes| {
            remotes
                .into_iter()
                .map(|(url, remote)| {
                    let (service_name, display_name) =
                        if let Some(service) = RemoteService::from_url(&url) {
                            (
                                Some(service.cli_name().to_string()),
                                Some(service.display_name_plural().to_string()),
                            )
                        } else {
                            (None, None)
                        };

                    // Get watch details if watch_id exists
                    let (watch_status, watch_status_summary, watch_last_error, current_pr) =
                        if let Some(watch_id_str) = &remote.watch_id {
                            if let Ok(watch_id) = watch_id_str.parse::<u64>() {
                                if let Some(details) = watch_details_map.get(&watch_id) {
                                    let status = Some(details.status.to_string());
                                    let summary = if !details.status_details.summary.is_empty() {
                                        Some(details.status_details.summary.clone())
                                    } else {
                                        None
                                    };
                                    let error = details.status_details.last_error.clone();
                                    let pr = details.status_details.current_pr.as_ref().map(|pr| {
                                        PullRequestInfo {
                                            status: pr.status.clone(),
                                            url: pr.url.clone(),
                                        }
                                    });
                                    (status, summary, error, pr)
                                } else {
                                    (None, None, None, None)
                                }
                            } else {
                                (None, None, None, None)
                            }
                        } else {
                            (None, None, None, None)
                        };

                    // Get the actual status calculated by remote_statuses()
                    let status = remote_statuses.get(&url).map(|(_, status)| *status);

                    let enriched_remote = EnrichedDocumentRemote {
                        pulled_at: remote.pulled_at,
                        pushed_at: remote.pushed_at,
                        watch_id: remote.watch_id,
                        watch_direction: remote.watch_direction,
                        service_name,
                        display_name,
                        status,
                        watch_status,
                        watch_status_summary,
                        watch_last_error,
                        current_pr,
                    };

                    (url.to_string(), enriched_remote)
                })
                .collect()
        });

        let enriched = EnrichedDocumentTracking {
            id: tracking.id,
            cached_at: tracking.cached_at,
            added_at: tracking.added_at,
            remotes: enriched_remotes,
        };

        enriched_entries.insert(path, enriched);
    }

    enriched_entries
}

/// Custom LSP request to list document tracking
pub struct ListDocumentTracking;

impl Request for ListDocumentTracking {
    const METHOD: &'static str = "stencila.documents/tracking";
    type Params = ();
    type Result = EnrichedDocumentTrackingEntries;
}
