use std::{
    env::current_dir,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use cli_utils::{
    confirm,
    table::{self, Attribute, Cell, Color},
    AsFormat, Code, ToStdout,
};
use common::{
    chrono::TimeDelta,
    chrono_humanize,
    clap::{self, Parser},
    eyre::{bail, Report, Result},
    futures::future::try_join_all,
    reqwest::Url,
    tokio::fs::write,
};

use crate::track::{closest_workspace_dir, tracking_file, DocumentRemote};

use super::{track::DocumentTrackingStatus, Document};

/// Initialize document tracking in a folder
#[derive(Debug, Parser)]
pub struct Init {
    /// The directory to start document tracking in
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    dir: PathBuf,

    /// Do not create a `.gitignore` file
    #[arg(long)]
    no_gitignore: bool,
}

impl Init {
    pub async fn run(self) -> Result<()> {
        if !self.dir.exists() {
            bail!(
                "Workspace directory `{}` does not exist",
                self.dir.display()
            );
        }

        tracking_file(&self.dir, true).await?;

        if !self.no_gitignore {
            write(self.dir.join(".stencila").join(".gitignore"), "*\n").await?;
        }

        eprintln!(
            "🟢 Initialized document tracking in `{}`",
            self.dir.display()
        );

        Ok(())
    }
}

/// Start tracking a document
#[derive(Debug, Parser)]
pub struct Track {
    /// The path to the local file to track
    file: PathBuf,

    /// The URL of the remote to track
    url: Option<Url>,
}

impl Track {
    pub async fn run(self) -> Result<()> {
        if let Some(url) = self.url {
            let already_tracked =
                Document::track_remote(&self.file, (url.clone(), DocumentRemote::default()))
                    .await?;
            eprintln!(
                "🟢 {} tracking {url} for `{}`",
                if already_tracked {
                    "Continued"
                } else {
                    "Started"
                },
                self.file.display()
            );
        } else {
            let (already_tracked, ..) = Document::track_path(&self.file, None).await?;
            eprintln!(
                "🟢 {} tracking `{}`",
                if already_tracked {
                    "Continued"
                } else {
                    "Started"
                },
                self.file.display()
            );
        }

        Ok(())
    }
}

/// Stop tracking a document
#[derive(Debug, Parser)]
pub struct Untrack {
    /// The path of the file to stop tracking
    file: PathBuf,

    /// The URL of the remote to stop tracking
    url: Option<Url>,
}

impl Untrack {
    pub async fn run(self) -> Result<()> {
        if let Some(url) = self.url {
            Document::untrack_remote(&self.file, &url).await?;
            eprintln!("🟥 Stopped tracking {url} for `{}`", self.file.display());
        } else {
            Document::untrack_path(&self.file).await?;
            eprintln!("🟥 Stopped tracking `{}`", self.file.display());
        }

        Ok(())
    }
}

/// Move a tracked document
///
/// Moves the document file to the new path (if it still exists at the
/// old path) and updates any tracking information.
#[derive(Debug, Parser)]
#[clap(alias = "mv")]
pub struct Move {
    /// The old path of the file
    from: PathBuf,

    /// The new path of the file
    to: PathBuf,

    /// Overwrite the destination path if it already exists
    #[arg(long, short)]
    force: bool,
}

impl Move {
    pub async fn run(self) -> Result<()> {
        if self.to.exists()
            && !self.force
            && !confirm("Destination path already exists, overwrite it?")?
        {
            return Ok(());
        }

        Document::move_path(&self.from, &self.to).await
    }
}

/// Remove a tracked document
///
/// Deletes the document file (if it still exists) and removes
/// any tracking data from the `.stencila` directory.
#[derive(Debug, Parser)]
#[clap(alias = "rm")]
pub struct Remove {
    /// The path of the file to remove
    file: PathBuf,

    /// Do not ask for confirmation of removal
    #[arg(long, short)]
    force: bool,
}

impl Remove {
    pub async fn run(self) -> Result<()> {
        if self.file.exists()
            && !self.force
            && !confirm(&format!(
                "Are you sure you want to remove {}?",
                self.file.display()
            ))?
        {
            return Ok(());
        }

        Document::remove_path(&self.file).await
    }
}

/// Get the tracking status of documents
#[derive(Debug, Parser)]
pub struct Status {
    /// The paths of the files to get status for
    files: Vec<PathBuf>,

    /// Output the status as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl Status {
    pub async fn run(self) -> Result<()> {
        let statuses = if self.files.is_empty() {
            // No paths provided, so get statuses from tracking dir
            match Document::tracking_all(&current_dir()?).await? {
                Some(statuses) => statuses,
                None => {
                    eprintln!("⚪️ Path is not in a folder being tracked by Stencila");
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

            // Get status of each file
            let futures = self.files.into_iter().map(|path| async {
                let status = Document::tracking_path(&path).await?;
                Ok::<_, Report>((path, status))
            });
            let statuses = try_join_all(futures).await?;
            statuses
                .into_iter()
                .flat_map(|(path, tracking)| {
                    tracking.and_then(|tracking| tracking.1.map(|entry| (path, entry)))
                })
                .collect()
        };

        if let Some(format) = self.r#as {
            // Return early with formatted list
            Code::new_from(format.into(), &statuses)?.to_stdout();
            return Ok(());
        }

        let workspace_dir = closest_workspace_dir(&current_dir()?, false).await?;

        let mut table = table::new();
        table.set_header([
            "File\n↳ Remote",
            "Status",
            "Modified\n↳ Pulled",
            "Stored\n↳ Pushed",
        ]);

        for (path, entry) in statuses {
            let (status, modified_at) = entry.status(&workspace_dir, &path);

            use DocumentTrackingStatus::*;
            let (attr, color) = match status {
                Unsupported => (Attribute::Dim, Color::DarkGrey),
                Deleted => (Attribute::Bold, Color::Red),
                Synced => (Attribute::Bold, Color::Green),
                Ahead => (Attribute::Bold, Color::Yellow),
                Behind => (Attribute::Bold, Color::Red),
            };

            table.add_row([
                Cell::new(path.to_string_lossy()).add_attribute(attr),
                // Currently, only show status for deleted files
                Cell::new(if matches!(status, DocumentTrackingStatus::Deleted) {
                    status.to_string()
                } else {
                    String::new()
                })
                .fg(color),
                // Do not show time if deleted
                Cell::new(if matches!(status, DocumentTrackingStatus::Deleted) {
                    String::new()
                } else {
                    humanize_timestamp(modified_at)?
                }),
                Cell::new(humanize_timestamp(entry.stored_at)?),
            ]);

            for (url, remote) in entry.remotes.iter().flatten() {
                table.add_row([
                    Cell::new(format!("↳ {url}")),
                    Cell::new("").fg(color),
                    Cell::new(format!("↳ {}", humanize_timestamp(remote.pulled_at)?))
                        .add_attribute(if remote.pulled_at.is_none() {
                            Attribute::Dim
                        } else {
                            Attribute::Reset
                        }),
                    Cell::new(format!("↳ {}", humanize_timestamp(remote.pushed_at)?))
                        .add_attribute(if remote.pushed_at.is_none() {
                            Attribute::Dim
                        } else {
                            Attribute::Reset
                        }),
                ]);
            }
        }

        table.to_stdout();

        Ok(())
    }
}

fn humanize_timestamp(time: Option<u64>) -> Result<String> {
    use chrono_humanize::{Accuracy, HumanTime, Tense};

    let Some(time) = time else {
        return Ok(String::from("never"));
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
