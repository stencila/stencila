use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env::current_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::TimeDelta;
use clap::Parser;
use eyre::{Result, bail};
use inflector::Inflector;
use reqwest::Url;
use stencila_cloud::DirectionState;

use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    tabulated::{Attribute, Cell, CellAlignment, Color, Tabulated},
};
use stencila_codec_utils::git_info;
use stencila_dirs::closest_workspace_dir;
use stencila_remotes::{
    RemoteService, RemoteStatus, calculate_remote_statuses, get_all_remote_entries,
    get_remotes_for_path, remove_deleted_watches,
};

/// Get the tracking status of documents
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The paths of the files to get status for
    files: Vec<PathBuf>,

    /// Output the status as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,

    /// Skip fetching remote status
    #[arg(long)]
    no_remotes: bool,

    /// Skip fetching watch status
    #[arg(long)]
    no_watches: bool,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show status of all tracked documents (includes watch details by default)</dim>
  <b>stencila status</>

  <dim># Show status of specific documents</dim>
  <b>stencila status</> <g>document.md</> <g>report.md</>

  <dim># Output status as JSON</dim>
  <b>stencila status</> <c>--as</> <g>json</>

  <dim># Skip fetching remote status (faster)</dim>
  <b>stencila status</> <c>--no-remotes</>

  <dim># Skip fetching watch status (faster)</dim>
  <b>stencila status</> <c>--no-watches</>
"
);

impl Cli {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        // Use workspace root (not CWD) so paths are resolved correctly regardless of where command is run
        let workspace_dir = closest_workspace_dir(&current_dir()?, false).await?;

        let file_entries = if self.files.is_empty() {
            // No paths provided, get all tracked files from config
            match get_all_remote_entries(&workspace_dir).await? {
                Some(entries) => entries,
                None => {
                    message!("✖️  No remotes configured in `stencila.toml`");
                    return Ok(());
                }
            }
        } else {
            // Check that each path exists
            for path in self.files.iter() {
                if !path.exists() {
                    bail!("Path does not exist: {}", path.display())
                }
            }

            // Get remotes for specified files
            let mut entries = stencila_remotes::RemoteEntries::new();
            for path in self.files.iter() {
                let remotes: BTreeMap<Url, stencila_remotes::RemoteInfo> =
                    get_remotes_for_path(path, Some(&workspace_dir))
                        .await?
                        .into_iter()
                        .map(|r| (r.url.clone(), r))
                        .collect();

                if !remotes.is_empty() {
                    entries.insert(path.clone(), remotes);
                }
            }
            entries
        };

        if let Some(format) = self.r#as {
            // Return early with formatted list
            Code::new_from(format.into(), &file_entries)?.to_stdout();
            return Ok(());
        }

        // Get repo URL for filtering watches (if in a git repository)
        let repo_url = git_info(&workspace_dir).ok().and_then(|info| info.origin);

        // Fetch watch details from API if not skipping remotes or watches
        let (watch_details_map, removed_watches): (
            HashMap<u64, stencila_cloud::WatchDetailsResponse>,
            Vec<_>,
        ) = if !self.no_watches {
            match stencila_cloud::get_watches(repo_url.as_deref()).await {
                Ok(watches) => {
                    let watch_map: HashMap<u64, stencila_cloud::WatchDetailsResponse> =
                        watches.into_iter().map(|w| (w.id, w)).collect();

                    // Clean up watch_ids that no longer exist in the cloud
                    let valid_watch_ids: HashSet<u64> = watch_map.keys().copied().collect();
                    let removed_watches =
                        match remove_deleted_watches(&current_dir()?, &valid_watch_ids).await {
                            Ok(removed) => removed,
                            Err(error) => {
                                tracing::warn!("Failed to cleanup deleted watches: {error}");
                                Vec::new()
                            }
                        };

                    (watch_map, removed_watches)
                }
                Err(error) => {
                    tracing::debug!("Failed to fetch watch details from API: {error}");
                    (HashMap::new(), Vec::new())
                }
            }
        } else {
            (HashMap::new(), Vec::new())
        };

        // Collect watch details for later display if --watch-details is enabled
        let mut watch_details_for_display = Vec::new();

        let mut table = Tabulated::new();
        table.set_header([
            "File/Remote",
            "Status",
            "Modified",
            "Pulled",
            "Pushed",
            "Watch",
        ]);

        // Track statuses that appear in the table for legend
        let mut seen_statuses = HashSet::new();

        // Track whether any remotes were displayed
        let mut has_remotes = false;

        // Create a set of removed watch (path, url) pairs for quick lookup
        let removed_watch_set: HashSet<(PathBuf, Url)> = removed_watches
            .into_iter()
            .map(|(path, url, _)| (path, url))
            .collect();

        for (path, entry) in file_entries {
            // Calculate file modification time for status comparison
            let (file_status, modified_at) = if path.exists() {
                let modified = path
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()))
                    .unwrap_or(0);
                (RemoteStatus::Unknown, Some(modified))
            } else {
                (RemoteStatus::Deleted, None)
            };

            use RemoteStatus::*;

            let status_color = |status: &RemoteStatus| match status {
                Unknown => Color::DarkGrey,
                Deleted => Color::Red,
                Diverged => Color::Magenta,
                Behind => Color::Yellow,
                Synced => Color::Green,
                Ahead => Color::Cyan,
            };

            // Fetch remote statuses in parallel (unless --no-remotes flag is set)
            let remote_statuses = if self.no_remotes {
                BTreeMap::new()
            } else {
                calculate_remote_statuses(&entry, file_status, modified_at).await
            };

            // Only show remotes (no local file row)
            for (url, remote) in &entry {
                // Mark that we have at least one remote
                has_remotes = true;

                // Helper function to get service name from URL
                let service_name = |url: &Url| -> String {
                    RemoteService::from_url(url)
                        .map(|s| s.display_name_plural().to_string())
                        .unwrap_or_else(|| url.to_string())
                };

                // Get remote status and modified time from fetched metadata
                let (remote_modified_at, remote_status) = remote_statuses
                    .get(url)
                    .cloned()
                    .unwrap_or((None, RemoteStatus::Unknown));

                // Track remote status for legend
                if !matches!(remote_status, Unknown) {
                    seen_statuses.insert(remote_status);
                }

                // Format watch status with directional arrows and colors
                // Check if this watch was removed
                let watch_cell = if removed_watch_set.contains(&(path.clone(), url.clone())) {
                    Cell::new("Removed")
                        .fg(Color::DarkGrey)
                        .add_attribute(Attribute::Dim)
                } else if let Some(watch_id) = remote
                    .watch_id
                    .as_ref()
                    .and_then(|id| id.parse::<u64>().ok())
                {
                    use stencila_remotes::WatchDirection;
                    let direction = remote.watch_direction.unwrap_or_default();

                    // Get watch details from API if available
                    let (watch_status_color, watch_status_text) = watch_details_map
                        .get(&watch_id)
                        .map(|details| {
                            use stencila_cloud::WatchStatus;
                            let color = match details.status {
                                WatchStatus::Ok => Color::Green,
                                WatchStatus::Pending => Color::Yellow,
                                WatchStatus::Syncing => Color::Cyan,
                                WatchStatus::Blocked => Color::Magenta,
                                WatchStatus::Error => Color::Red,
                            };
                            let text = details.status.to_string();
                            (color, text)
                        })
                        .unzip();

                    // Collect watch details for display (unless --no-watch-details is set)
                    if !self.no_watches
                        && let Some(details) = watch_details_map.get(&watch_id)
                    {
                        watch_details_for_display.push((
                            path.clone(),
                            url.clone(),
                            details.clone(),
                        ));
                    }

                    let direction_str = match direction {
                        WatchDirection::Bi => "↔",
                        WatchDirection::FromRemote => "←",
                        WatchDirection::ToRemote => "→",
                    };

                    // Build the display string with status if available
                    let display_str = if let Some(status) = watch_status_text {
                        format!("{} {}", direction_str, status)
                    } else {
                        direction_str.to_string()
                    };

                    // Use watch status color if available, otherwise use direction default color
                    let color = watch_status_color.unwrap_or(match direction {
                        WatchDirection::Bi => Color::Green,
                        WatchDirection::FromRemote => Color::Yellow,
                        WatchDirection::ToRemote => Color::Cyan,
                    });

                    Cell::new(display_str).fg(color)
                } else {
                    Cell::new("-").fg(Color::DarkGrey)
                };

                table.add_row([
                    // Remote name
                    Cell::new(format!("└ {}", service_name(url))),
                    // Remote status
                    Cell::new(if matches!(remote_status, RemoteStatus::Unknown) {
                        String::new()
                    } else {
                        remote_status.to_string()
                    })
                    .fg(status_color(&remote_status)),
                    // Remote modification time
                    Cell::new(humanize_timestamp(remote_modified_at)?)
                        .set_alignment(CellAlignment::Right),
                    // Pulled time
                    Cell::new((humanize_timestamp(remote.pulled_at)?).to_string())
                        .add_attribute(if remote.pulled_at.is_none() {
                            Attribute::Dim
                        } else {
                            Attribute::Reset
                        })
                        .set_alignment(CellAlignment::Right),
                    // Pushed time
                    Cell::new((humanize_timestamp(remote.pushed_at)?).to_string())
                        .add_attribute(if remote.pushed_at.is_none() {
                            Attribute::Dim
                        } else {
                            Attribute::Reset
                        })
                        .set_alignment(CellAlignment::Right),
                    // Watch status
                    watch_cell,
                ]);
            }
        }

        table.to_stdout();

        // Print note only if there were any remotes
        if has_remotes {
            message!("Modification time updates for remotes can be delayed by 1-3 minutes.");
        }

        // Print legend if any non-Unknown statuses were displayed
        if !seen_statuses.is_empty() {
            use RemoteStatus::*;

            let mut parts = Vec::new();

            if seen_statuses.contains(&Ahead) {
                parts.push(cstr!(
                    "<cyan>Ahead</>: run *stencila pull <<file>>* to merge remote changes into local."
                ));
            }
            if seen_statuses.contains(&Behind) {
                parts.push(cstr!(
                    "<yellow>Behind</>: run *stencila push <<file>>* to upload local changes to remote."
                ));
            }
            if seen_statuses.contains(&Diverged) {
                parts.push(cstr!("<magenta>Diverged</>: run *stencila pull <<file>>* to create a local branch and merge remote changes."));
            }
            if seen_statuses.contains(&Deleted) {
                parts.push(cstr!(
                    "<red>Deleted</>: run *stencila untrack <<file>>* to stop tracking deleted file."
                ));
            }

            message!("{}", parts.join("\n"));
        }

        // Display detailed watch information (unless --no-watch-details is set)
        if !self.no_watches && !watch_details_for_display.is_empty() {
            let direction_state_color = |state| match state {
                DirectionState::Ok => Color::Green,
                DirectionState::Pending => Color::Yellow,
                DirectionState::Running => Color::Cyan,
                DirectionState::Blocked => Color::Magenta,
                DirectionState::Error => Color::Red,
                DirectionState::Disabled => Color::DarkGrey,
            };

            for (file_path, remote_url, details) in watch_details_for_display {
                eprintln!();

                // Create a separate table for this watch
                let mut watch_table = Tabulated::new();
                watch_table.set_header([
                    "Watch", "Status", "Received", "Started", "Finished", "Reason",
                ]);

                // Determine service name for display
                let service_name = RemoteService::from_url(&remote_url)
                    .map(|s| s.display_name_plural().to_string())
                    .unwrap_or_else(|| remote_url.to_string());

                // Add header row for this watch
                watch_table.add_row([
                    Cell::new(file_path.display()).add_attribute(Attribute::Bold),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                ]);

                // Add row for to_remote direction if present
                if let Some(to_remote) = &details.status_details.directions.to_remote {
                    let state_color = direction_state_color(to_remote.state);
                    let state_text = to_remote.state.to_string();

                    let received = to_remote
                        .last_received_at
                        .as_ref()
                        .map(|t| format_timestamp(t))
                        .unwrap_or_else(|| "-".to_string());

                    let queued = to_remote
                        .last_queued_at
                        .as_ref()
                        .map(|t| format_timestamp(t))
                        .unwrap_or_else(|| "-".to_string());

                    let processed = to_remote
                        .last_processed_at
                        .as_ref()
                        .map(|t| format_timestamp(t))
                        .unwrap_or_else(|| "-".to_string());

                    let reason = format_reason(&to_remote.reason, &to_remote.recommended_action);

                    watch_table.add_row([
                        Cell::new(format!("└ To {service_name}")),
                        Cell::new(state_text).fg(state_color),
                        Cell::new(received),
                        Cell::new(queued),
                        Cell::new(processed),
                        Cell::new(reason),
                    ]);
                }

                // Add row for from_remote direction if present
                if let Some(from_remote) = &details.status_details.directions.from_remote {
                    let state_color = direction_state_color(from_remote.state);
                    let state_text = from_remote.state.to_string();

                    let received = from_remote
                        .last_received_at
                        .as_ref()
                        .map(|t| format_timestamp(t))
                        .unwrap_or_else(|| "-".to_string());

                    let queued = from_remote
                        .last_queued_at
                        .as_ref()
                        .map(|t| format_timestamp(t))
                        .unwrap_or_else(|| "-".to_string());

                    let processed = from_remote
                        .last_processed_at
                        .as_ref()
                        .map(|t| format_timestamp(t))
                        .unwrap_or_else(|| "-".to_string());

                    let reason =
                        format_reason(&from_remote.reason, &from_remote.recommended_action);

                    watch_table.add_row([
                        Cell::new(format!("└ From {service_name}")),
                        Cell::new(state_text).fg(state_color),
                        Cell::new(received),
                        Cell::new(queued),
                        Cell::new(processed),
                        Cell::new(reason),
                    ]);
                }

                watch_table.to_stdout();

                // Display summary message after the table
                let mut message_parts = Vec::new();

                // Add summary
                if !details.status_details.summary.is_empty() {
                    message_parts.push(details.status_details.summary.clone());
                }

                // Add error in red if present
                if let Some(error) = &details.status_details.last_error {
                    message_parts.push(format!("{} {}", cstr!("<red>Error:</>"), error));
                }

                // Add recommended actions if present
                if let Some(actions) = details.status_details.recommended_actions
                    && !actions.is_empty()
                {
                    message_parts.extend(actions);
                }

                // Add link to PR is there is any
                if let Some(pr) = details.status_details.current_pr {
                    let pr = format!(
                        "Current {service_name} to repo PR is `{}`: {}",
                        pr.status, pr.url
                    );
                    message_parts.push(pr);
                }

                // Print the combined message
                if !message_parts.is_empty() {
                    message!("{}", message_parts.join("\n"));
                }
            }
        }

        Ok(())
    }
}

/// Format an ISO 8601 timestamp to a human-readable relative time
fn format_timestamp(iso_timestamp: &str) -> String {
    use chrono::{DateTime, Utc};
    use chrono_humanize::{Accuracy, HumanTime, Tense};

    if let Ok(dt) = DateTime::parse_from_rfc3339(iso_timestamp) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&Utc));

        if let Ok(time_delta) = TimeDelta::from_std(duration.to_std().unwrap_or_default()) {
            let mut string =
                HumanTime::from(time_delta).to_text_en(Accuracy::Rough, Tense::Present);
            if string == "now" {
                string.insert_str(0, "just ");
            } else {
                string.push_str(" ago");
            }
            return string;
        }
    }

    // Fallback to showing the raw timestamp if parsing fails
    iso_timestamp.to_string()
}

/// Format reason and recommended action for display
fn format_reason(reason: &Option<String>, recommended_action: &Option<String>) -> String {
    let reason_text = reason.as_ref().map(|r| r.to_sentence_case());
    let action_text = recommended_action.as_deref();

    match (reason_text, action_text) {
        (Some(r), Some(a)) => format!("{}. {}", r, a),
        (Some(r), None) => r,
        (None, Some(a)) => a.to_string(),
        (None, None) => "-".to_string(),
    }
}

fn humanize_timestamp(time: Option<u64>) -> Result<String> {
    use chrono_humanize::{Accuracy, HumanTime, Tense};

    let Some(time) = time else {
        return Ok(String::from("-"));
    };

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .saturating_sub(time);
    let time_delta = TimeDelta::seconds(seconds as i64);

    let mut string = HumanTime::from(time_delta).to_text_en(Accuracy::Rough, Tense::Present);
    if string == "now" {
        string.insert_str(0, "just ");
    } else {
        string.push_str(" ago");
    }

    Ok(string)
}
