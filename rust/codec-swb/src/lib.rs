use std::{fs::File, path::Path};

use flate2::{write::GzEncoder, Compression};

use codec::{
    common::{
        async_trait::async_trait,
        clap::{self, Parser},
        eyre::{Ok, Result},
        tar::Builder,
        tempfile::TempDir,
        tokio::fs::write,
    },
    format::Format,
    schema::Node,
    status::Status,
    Codec, CodecSupport, EncodeInfo, EncodeOptions,
};
use codec_dom::DomCodec;
use codec_jsonld::JsonLdCodec;
use web_dist::Web;

/// A codec for creating a Stencila Web Bundle (SWB)
///
/// A SWB is simply a `tar.gz` of the files and folders needed
/// to publish a document or several documents on the web, including
/// images, fonts, and JavaScript.
///
/// Each folder in the SWB, including the root, normally has an
/// `index.html` that is generated from the "main" document in
/// that folder.
#[derive(Debug, Default, Parser)]
pub struct SwbCodec {
    /// Do not publish a HTML file
    #[arg(long)]
    no_html: bool,

    /// Do not publish a JSON-LD file
    #[arg(long)]
    no_jsonld: bool,

    /// Disallow all bots
    #[arg(long)]
    no_bots: bool,

    /// Disallow AI bots
    #[arg(long, conflicts_with = "no_bots")]
    no_ai_bots: bool,
}

#[async_trait]
impl Codec for SwbCodec {
    fn name(&self) -> &str {
        "swb"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Swb => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let options = options.unwrap_or_default();

        // Create a temp dir to put all files for the bundle
        let temp_dir = TempDir::new()?;

        if !self.no_html {
            // Create the index.html file
            let html = temp_dir.path().join("index.html");

            let mut alternates = Vec::new();
            if !self.no_jsonld {
                alternates.push((
                    "application/ld+json".to_string(),
                    "index.jsonld".to_string(),
                ));
            }

            DomCodec {}
                .to_path(
                    node,
                    &html,
                    Some(EncodeOptions {
                        alternates: Some(alternates),
                        ..options.clone()
                    }),
                )
                .await?;

            // Add web dist to `~static`
            let statics = temp_dir.path().join("~static");
            Web::to_path(&statics, true)?;
        }

        if !self.no_jsonld {
            // Create JSON-LD file
            let jsonld = temp_dir.path().join("index.jsonld");
            JsonLdCodec {}
                .to_path(node, &jsonld, Some(options.clone()))
                .await?;
        }

        if self.no_bots || self.no_ai_bots {
            // Create robots.txt file
            let content = if self.no_bots {
                include_str!("all.robots.txt")
            } else {
                include_str!("ai.robots.txt")
            };
            let robots = temp_dir.path().join("robots.txt");
            write(robots, content).await?;
        }

        // Create a tar.gz archive of temp dir
        let tar_gz = File::create(path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);
        tar.append_dir_all(".", temp_dir.path())?;
        tar.finish()?;

        Ok(EncodeInfo::none())
    }
}
