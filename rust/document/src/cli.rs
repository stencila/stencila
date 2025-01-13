use std::{
    env::current_dir,
    fs::create_dir_all,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use cli_utils::{
    table::{self, Attribute, Cell, Color},
    AsFormat, Code, ToStdout,
};
use common::{
    chrono::TimeDelta,
    chrono_humanize,
    clap::{self, Parser},
    eyre::{bail, Result},
    futures::future::try_join_all,
    indexmap::IndexMap,
    itertools::Itertools,
};

use crate::track::DocumentStatus;

use super::{track::DocumentStatusFlag, Document};

/// Initialize tracking in a folder
#[derive(Debug, Parser)]
pub struct Init {
    /// The directory to start document tracking in
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    dir: PathBuf,
}

impl Init {
    pub async fn run(self) -> Result<()> {
        let path = self.dir.canonicalize()?.join(".stencila").join("tracked");

        if !path.exists() {
            create_dir_all(&path)?
        }

        Ok(())
    }
}

/// Start tracking a document
#[derive(Debug, Parser)]
pub struct Track {
    /// The path of the file to track
    path: PathBuf,
}

impl Track {
    pub async fn run(self) -> Result<()> {
        Document::track_path(&self.path).await
    }
}

/// Stop tracking a document
#[derive(Debug, Parser)]
pub struct Untrack {
    /// The path of the file to track
    path: PathBuf,
}

impl Untrack {
    pub async fn run(self) -> Result<()> {
        Document::untrack_path(&self.path).await
    }
}

/// Get the tracking status documents
#[derive(Debug, Parser)]
pub struct Status {
    /// The paths of the files to get status for
    paths: Vec<PathBuf>,

    /// Output the status as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl Status {
    pub async fn run(self) -> Result<()> {
        let statuses = if self.paths.is_empty() {
            // No paths provided, so get statuses from tracking dir
            Document::status_tracked(&current_dir()?).await?
        } else {
            // Check that each path exists
            for path in self.paths.iter() {
                if !path.exists() {
                    bail!("Path does not exist: {}", path.display())
                }
            }

            // Get status of each file
            let futures = self
                .paths
                .into_iter()
                .map(|path| Document::status_path(path, None));
            try_join_all(futures).await?
        };

        if let Some(format) = self.r#as {
            // Return early with formatted list
            Code::new_from(format.into(), &statuses)?.to_stdout();
            return Ok(());
        }

        // Find documents with more than two statuses so they can be associated in table
        let mut map: IndexMap<String, Vec<PathBuf>> = IndexMap::new();
        for status in &statuses {
            if let (Some(doc_id), Some(path)) = (&status.doc_id, &status.path) {
                map.entry(doc_id.to_string())
                    .or_insert_with(Vec::new)
                    .push(path.clone());
            }
        }
        let groups: Vec<String> = map
            .into_iter()
            .filter_map(|(doc_id, paths)| (paths.len() > 1).then_some(doc_id))
            .collect();

        let mut table = table::new();
        table.set_header(["File", "Linked", "Status", "Modified", "Tracked"]);

        for DocumentStatus {
            path,
            status,
            modified_at,
            tracked_at,
            doc_id,
        } in statuses
        {
            let path = path.unwrap_or_default().to_string_lossy().to_string();

            let linked = if let Some((group, ..)) =
                doc_id.and_then(|doc_id| groups.iter().find_position(|&id| id == &doc_id))
            {
                // Cycle through first 14 ANSI colors for groups
                let color = Color::AnsiValue(1 + (group as u8) % 14);
                Cell::new(format!("={group}")).fg(color)
            } else {
                Cell::new("")
            };

            use DocumentStatusFlag::*;
            let (attr, color) = match status {
                Unsupported => (Attribute::Dim, Color::DarkGrey),
                Untracked => (Attribute::Reset, Color::Blue),
                Unsaved => (Attribute::Dim, Color::DarkGrey),
                IdMissing(..) => (Attribute::Bold, Color::Magenta),
                IdDifferent(..) => (Attribute::Bold, Color::Magenta),
                Deleted => (Attribute::Bold, Color::Red),
                Synced => (Attribute::Reset, Color::Green),
                Ahead => (Attribute::Bold, Color::Yellow),
                Behind => (Attribute::Bold, Color::Red),
            };
            let status = match status {
                IdMissing(id) => format!("Missing id ({id})"),
                IdDifferent(id1, id2) => format!("Different ids ({id1} != {id2})"),
                _ => status.to_string(),
            };

            let modified_at = humanize_timestamp(modified_at)?;
            let tracked_at = humanize_timestamp(tracked_at)?;

            table.add_row([
                Cell::new(path).add_attribute(attr),
                linked,
                Cell::new(status).fg(color),
                Cell::new(modified_at),
                Cell::new(tracked_at),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

fn humanize_timestamp(time: Option<u64>) -> Result<String> {
    use chrono_humanize::{Accuracy, HumanTime, Tense};

    let Some(time) = time else {
        return Ok(String::from(""));
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
