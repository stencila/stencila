use std::{fs::File, path::Path};

use flate2::{write::GzEncoder, Compression};

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{Ok, Result},
        tar::Builder,
        tempfile::TempDir,
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
pub struct SwbCodec;

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

        // Create the index.html file
        let html = temp_dir.path().join("index.html");
        let alternates = Some(vec![(
            "application/ld+json".to_string(),
            "index.jsonld".to_string(),
        )]);
        DomCodec {}
            .to_path(
                node,
                &html,
                Some(EncodeOptions {
                    alternates,
                    ..options.clone()
                }),
            )
            .await?;

        // Create JSON-LD file for index
        let jsonld = temp_dir.path().join("index.jsonld");
        JsonLdCodec {}
            .to_path(node, &jsonld, Some(EncodeOptions { ..options }))
            .await?;

        // Add web dist to `~static`
        let statics = temp_dir.path().join("~static");
        Web::to_path(&statics, true)?;

        // Create a tar.gz archive of temp dir
        let tar_gz = File::create(&path)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);
        tar.append_dir_all(".", temp_dir.path())?;
        tar.finish()?;

        Ok(EncodeInfo::none())
    }
}
