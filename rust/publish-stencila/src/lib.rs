use std::path::{Path, PathBuf};

use cloud::ErrorResponse;
use codec::{Codec, EncodeOptions};
use codec_swb::SwbCodec;
use common::{
    clap::{self, Parser},
    eyre::{bail, eyre, Result},
    reqwest::{
        multipart::{Form, Part},
        Client,
    },
    serde::Serialize,
    serde_json,
    tempfile::TempDir,
    tokio, tracing,
};
use document::{CommandWait, Document};
use schema::Node;

/// Publish to Stencila Cloud
#[derive(Debug, Parser)]
pub struct Cli {
    /// Path to the file or directory to publish
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// The key for the site
    #[arg(long, short)]
    key: Option<String>,

    /// Perform a dry run only
    #[arg(long)]
    dry_run: bool,

    #[clap(flatten)]
    swb: SwbCodec,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        publish_path(&self.path, &self.key, self.dry_run, &self.swb).await
    }
}

/// Publish a path (file or directory)
async fn publish_path(
    path: &Path,
    key: &Option<String>,
    dry_run: bool,
    swb: &SwbCodec,
) -> Result<()> {
    if !path.exists() {
        bail!("Path does not exist: {}", path.display())
    }

    if path.is_file() {
        let doc = Document::open(path).await?;
        doc.compile(CommandWait::Yes).await?;

        let theme = doc.config().await?.theme;
        let node = &*doc.root_read().await;

        let options = EncodeOptions {
            theme,
            ..Default::default()
        };

        publish_node(node, options, key, dry_run, swb).await
    } else {
        bail!("Publishing of directories is not currently supported")
    }
}

#[derive(Serialize)]
#[serde(crate = "common::serde")]
struct Manifest {}

/// Publish a single node to Stencila Cloud
async fn publish_node(
    node: &Node,
    options: EncodeOptions,
    key: &Option<String>,
    dry_run: bool,
    swb: &SwbCodec,
) -> Result<()> {
    let token = cloud::api_key().ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or key chain entry found. Get one at https://stencila.cloud/."))?;

    let key = key.as_deref().unwrap_or_default().to_string();
    let base_url = format!("https://{key}.stencila.site");

    let manifest = Manifest {};
    let manifest = serde_json::to_string(&manifest)?;
    let manifest = Part::text(manifest);

    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().join("publish.swb");
    swb.to_path(
        node,
        &temp_path,
        Some(EncodeOptions {
            base_url: Some(base_url),
            ..options
        }),
    )
    .await?;

    let bundle: Vec<u8> = tokio::fs::read(temp_path).await?;
    let bundle = Part::bytes(bundle).file_name("publish.swb");

    let form = Form::new()
        .part("manifest", manifest)
        .part("bundle", bundle);

    if dry_run {
        tracing::info!("Dry run completed");
        return Ok(());
    }

    let response = Client::new()
        .put(format!("{}/sites/{}", cloud::base_url(), key))
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let ErrorResponse { error, .. } = response.json().await?;
        bail!("{error}")
    }
}
