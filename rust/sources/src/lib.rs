use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use common::{
    defaults::Defaults,
    eyre::{bail, Result},
    futures,
    futures::future,
    serde::{Deserialize, Serialize},
    serde_json,
    serde_with::skip_serializing_none,
    tokio::{self, sync::mpsc},
    tracing,
};
use files::{File, Files};
use graph_triples::{
    relations::{self, NULL_RANGE},
    resources, Resource, Triple,
};
use providers::provider::{ImportOptions, WatchMode};
use stencila_schema::Node;

/// A source-destination combination
///
/// Each source by destination combination should be unique to a project.
/// It is possible to have the same source being imported to multiple
/// destinations within a project and for multiple sources to used the same
/// destination (e.g. the root directory of the project).
#[skip_serializing_none]
#[derive(Clone, Debug, Defaults, Deserialize, Serialize)]
#[serde(default, crate = "common::serde")]
pub struct Source {
    /// The name of the source
    ///
    /// Useful for providing a shorthand way to refer to the source rather than using it's URL
    pub name: Option<String>,

    /// The URL of the source
    pub url: String,

    /// The providers that matched the URL
    pub provider: Option<String>,

    /// The node parsed from the URL by the provider
    pub node: Option<Node>,

    /// The destination path within the project
    pub dest: Option<PathBuf>,

    /// Run a cron schedule to import and/or export the source
    pub cron: Option<SourceCron>,

    /// Synchronize the source
    pub watch: Option<SourceWatch>,

    /// The name of the secret required to access the source
    ///
    /// To improve the security of API access tokens, secrets are only ever read from
    /// environment variables. Source providers usually have a default secret name
    /// e.g. `GITHUB_TOKEN`. However, this field allows setting of custom secret names
    /// which may be necessary, for example, if a project uses two sources from the
    /// same provider, requiring different secrets.
    pub secret_name: Option<String>,

    /// A list of file paths currently associated with the source (relative to the project root)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<PathBuf>,

    /// A running cron task
    #[serde(skip)]
    #[def = "None"]
    cron_task: Option<SourceTask>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, crate = "common::serde")]
pub struct SourceCron {
    /// The schedule on which to perform the action
    ///
    /// Can be a cron phrase (e.g. "every 10mins") or cron expression (e.g. "0 0 */10 * * *").
    schedule: String,

    /// The cron expression/s parsed from the `schedule`
    expressions: Option<Vec<String>>,

    /// The timezone parsed from the `schedule`
    timezone: Option<String>,

    /// The action to perform at each scheduled time
    action: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
pub struct SourceWatch {
    /// The synchronization mode
    mode: Option<WatchMode>,
}

#[derive(Debug, Clone)]
struct SourceTask {
    /// The channel sender used to cancel the source task
    ///
    /// Note that if the source is dropped (ie. removed from a list of sources) with a running task
    /// then this canceller will be dropped and so the task will also end (an explicit `.send()`
    /// is not required).
    canceller: mpsc::Sender<()>,
}

impl Source {
    /// Create a new source
    ///
    /// Resolves a [`Provider`] for the source.
    /// Ensures that `dest` is a relative path and does not include any traversal (i.e. `..`).
    /// Parses the `cron` into an expression and timezone.
    pub async fn new(
        url: String,
        name: Option<String>,
        dest: Option<PathBuf>,
        cron: Option<String>,
        watch: Option<WatchMode>,
    ) -> Result<Source> {
        let (provider, node) = providers::resolve(&url).await?;

        if let Some(dest) = dest.as_ref() {
            if dest.is_absolute() {
                bail!("Source destination must be a relative path; try removing any leading slash (MacOS or Linux) or drive letter (e.g. `C:/` on Windows)")
            }
            if dest.to_string_lossy().contains("..") {
                bail!("Source destination must not have any path traversal (a.k.a directory climbing); try removing any `..` in the destination path")
            }
        }

        let cron = if let Some(schedule) = cron {
            let (schedules, timezone) = cron_utils::parse(&schedule)?;
            let (expressions, timezone) = if !schedules.is_empty() {
                (
                    Some(
                        schedules
                            .iter()
                            .map(|schedule| schedule.to_string())
                            .collect(),
                    ),
                    timezone.map(|tz| tz.to_string())
                )
            } else {
                (None, None)
            };

            Some(SourceCron {
                schedule,
                expressions,
                timezone,
                action: None,
            })
        } else {
            None
        };

        let watch = watch.map(|mode| SourceWatch { mode: Some(mode) });

        Ok(Source {
            name,
            url,
            provider: Some(provider),
            node: Some(node),
            dest,
            cron,
            watch,
            ..Default::default()
        })
    }

    /// Generate a label for the source
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.url.clone())
    }

    /// Does the source match a string identifier
    ///
    /// Matches the string against the `name`, the `url` , or the filename (of the `dest` with or without extension).
    pub fn matches(&self, identifier: &str) -> bool {
        if let Some(name) = &self.name {
            if name == identifier {
                return true;
            }
        }

        if identifier == self.url {
            return true;
        }

        if let Some(dest) = &self.dest {
            if dest.to_string_lossy() == identifier {
                return true;
            }
            if let Some(file_stem) = dest.file_stem() {
                if file_stem == identifier {
                    return true;
                }
            }
        }

        false
    }

    /// Read a source from a JSON file
    ///
    /// Only changes the properties that are NOT saved in the project.json file.
    pub fn read<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            bail!("Project source file does not exist: {}", path.display())
        }
        let path = path.canonicalize()?;

        let json = fs::read_to_string(path)?;
        let source: Source = serde_json::from_str(&json)?;
        self.files = source.files;

        Ok(())
    }

    /// Create a graph resource for the source
    pub fn resource(&self) -> Resource {
        resources::url(&self.url)
    }

    /// Create a set of graph triples describing relation between the source it's associated files.
    pub fn triples(&self, project: &Path) -> Vec<Triple> {
        let this = self.resource();
        self.files
            .iter()
            .map(|file| {
                (
                    this.clone(),
                    relations::imports(NULL_RANGE),
                    resources::file(&project.join(file)),
                )
            })
            .collect()
    }

    /// Import the source
    ///
    /// # Arguments
    ///
    /// - `path`: The path to import the source into
    ///
    /// # Returns
    ///
    /// A map of the [`File`]s that were imported by the source.
    pub async fn import(&self, path: &Path) -> Result<BTreeMap<PathBuf, File>> {
        let (.., node) = providers::resolve(&self.url).await?;
        let dest = match &self.dest {
            Some(dest) => path.join(dest),
            None => path.to_path_buf(),
        };
        let options = ImportOptions {
            token: self.secret_name.clone(),
        };

        let (pre, ..) = Files::walk(path, false);

        providers::import(&node, &dest, Some(options)).await?;

        let (post, ..) = Files::walk(path, false);
        let files = post
            .into_iter()
            .filter_map(|(path, file)| match pre.get(&path) {
                None => Some((path, file)),
                Some(existing) => {
                    if existing.size != file.size && existing.modified != file.modified {
                        Some((path, file))
                    } else {
                        None
                    }
                }
            });

        Ok(files.collect())
    }

    /// Start a cron for the source
    pub async fn start(&mut self, dest: &Path) -> Result<()> {
        let cron = match (&self.cron, &self.cron_task) {
            (Some(cron), None) => cron,
            _ => return Ok(()),
        };

        tracing::info!("Starting cron task for source `{}`", self.label());
        let action = cron.action.clone().unwrap_or_default();
        let schedule = cron.schedule.clone();
        let (.., node) = providers::resolve(&self.url).await?;
        let dest = dest.to_path_buf();
        let (canceller, cancellee) = mpsc::channel(1);
        tokio::spawn(
            async move { providers::cron(&node, &dest, &action, &schedule, cancellee).await },
        );
        self.cron_task = Some(SourceTask { canceller });

        Ok(())
    }

    /// Stop the cron task for the source (if started)
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(task) = &self.cron_task {
            tracing::info!("Stopping cron task for source `{}`", self.label());
            task.canceller.send(()).await?
        }
        self.cron_task = None;

        Ok(())
    }
}

/// A set of sources, usually associated with a project
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(transparent, crate = "common::serde")]
pub struct Sources {
    /// The list of sources
    pub inner: Vec<Source>,

    /// If in a running state, the path that cron and watch tasks were started with
    ///
    /// If tasks are running then any new sources that are added will have
    /// tasks automatically started for them.
    #[serde(skip)]
    running: Option<PathBuf>,
}

impl Sources {
    /// Return a list of sources
    pub fn list(&self) -> Vec<String> {
        self.inner.iter().map(|source| source.label()).collect()
    }

    /// Find a source
    ///
    /// # Arguments
    ///
    /// - `identifier`: An identifier for a source.
    ///
    /// The first source that matches the identifier is returned. Will error if no match is found.
    pub fn find(&self, identifier: &str) -> Result<&Source> {
        if let Some(source) = self.inner.iter().find(|source| source.matches(identifier)) {
            return Ok(source);
        }

        bail!("Unable to find a source matching `{}`", identifier)
    }

    /// Add a source
    ///
    /// # Arguments
    ///
    /// - `source`: The source to add
    ///
    /// Warns if there is already a source with the same `url`.
    /// Errors if there is already a source with the same `name` or the same `url` and `dest`.
    ///
    /// If the `start` method has been called previously then the sources are in a "running"
    /// state and the `start` will be called on the source.
    pub async fn add(&mut self, mut source: Source) -> Result<()> {
        let source_dest = source.dest.clone().unwrap_or_else(|| PathBuf::from("."));
        for existing in self.inner.iter() {
            if let (Some(existing_name), Some(source_name)) = (&existing.name, &source.name) {
                if existing_name == source_name {
                    bail!("There is already a source with the name `{}`", source_name);
                }
            }
            let existing_dest = existing.dest.clone().unwrap_or_else(|| PathBuf::from("."));
            if existing.url == source.url && existing_dest == source_dest {
                bail!(
                    "There is already a source with the URL `{}` and destination `{}`",
                    source.url,
                    source_dest.display()
                );
            }
            if existing.url == source.url {
                tracing::warn!("There is already a source with the URL `{}`", source.url);
            }
        }
        if let Some(path) = &self.running {
            source.start(path).await?;
        }
        self.inner.push(source);

        Ok(())
    }

    /// Remove a source
    ///
    /// # Arguments
    ///
    /// - `identifier`: An identifier for a source.
    ///
    /// Any running tasks for the source will be stopped.
    ///
    /// The first source that matches the identifier is removed. See [`Source::matches`] for
    /// details. Will error if no match is found.
    pub async fn remove(&mut self, identifier: &str) -> Result<Source> {
        if let Some(index) = self
            .inner
            .iter()
            .position(|source| source.matches(identifier))
        {
            let mut removed = self.inner.remove(index);
            removed.stop().await?;
            return Ok(removed);
        }

        bail!("Unable to find a matching source to remove")
    }

    /// Import all sources
    ///
    /// # Arguments
    ///
    /// - `path`: The path to import the sources into (usually a project directory)
    ///
    /// Imports sources in parallel.
    pub async fn import(&self, path: &Path) -> Result<()> {
        let futures = self.inner.iter().map(|source| {
            let source = source.clone();
            let path = path.to_path_buf();
            tokio::spawn(async move { source.import(&path).await })
        });
        future::try_join_all(futures).await?;

        Ok(())
    }

    /// Import all sources synchronously
    ///
    /// # Arguments
    ///
    /// - `path`: The path to import the sources into (usually a project directory)
    ///
    /// Imports sources in parallel but blocks until all are complete.
    pub fn import_sync(&self, path: &Path) -> Result<()> {
        futures::executor::block_on(async { self.import(path).await })
    }

    /// Start cron and/or watch tasks for each source
    pub async fn start(&mut self, path: &Path) -> Result<()> {
        for source in self.inner.iter_mut() {
            let dest = match &source.dest {
                Some(dest) => path.join(dest),
                None => path.to_path_buf(),
            };
            source.start(&dest).await?;
        }
        self.running = Some(path.to_path_buf());

        Ok(())
    }

    /// Stop any cron or watch tasks for each source
    pub async fn stop(&mut self) -> Result<()> {
        for source in self.inner.iter_mut() {
            source.stop().await?;
        }
        self.running = None;

        Ok(())
    }
}
