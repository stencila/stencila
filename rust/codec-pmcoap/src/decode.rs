use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use codec::{
    common::{
        eyre::{bail, ContextCompat, OptionExt, Result},
        futures::StreamExt,
        glob::glob,
        regex::Regex,
        reqwest::Client,
        tar::Archive,
        tempfile,
        tokio::{fs::File, io::AsyncWriteExt},
        tracing,
    },
    schema::{Block, ImageObject, Inline, Node, VisitorMut, WalkControl, WalkNode},
    Codec, DecodeInfo, DecodeOptions,
};
use codec_jats::JatsCodec;
use flate2::read::GzDecoder;

/// Decode a PMCID to a Stencila [`Node`]
pub(super) async fn decode_id(
    id: &str,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    if !id.starts_with("PMC") {
        bail!("Unrecognized article id, should be a PMC id")
    }

    let dir = tempfile::TempDir::new()?;
    let dir = if cfg!(debug_assertions) {
        let dir = PathBuf::from("temp-pmc");
        create_dir_all(&dir)?;
        dir
    } else {
        dir.path().to_path_buf()
    };

    // Download and extract
    let dir = download_package(id, &dir).await?;

    // Find the JATS file in the dir
    let jats_path = glob(&dir.join("*.nxml").to_string_lossy())?
        .next()
        .and_then(|res| res.ok())
        .ok_or_eyre("Unable to find JATS XML file")?;

    // Decode the JATS
    let (mut node, info) = JatsCodec.from_path(&jats_path, options).await?;

    // Inline any images if possible
    node.walk_mut(&mut ImageInliner { dir });

    Ok((node, info))
}

/// Download the PMC OA Package for a PMCID
///
/// Returns the path to the directory of the extracted package.
pub(super) async fn download_package(pmcid: &str, dir: &Path) -> Result<PathBuf> {
    let pmcid = pmcid.trim();

    tracing::debug!("Getting URL for OA package for `{pmcid}`");
    let url = format!("https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi?id={pmcid}");
    let xml = Client::new()
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let re = Regex::new(r#"href="([^"]+\.tar\.gz)""#).unwrap();
    let ftp_url = re
        .captures(&xml)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .wrap_err("No .tar.gz link found")?;

    let https_url = ftp_url.replacen("ftp://", "https://", 1);

    tracing::debug!("Downloading {https_url}");
    let response = Client::new()
        .get(&https_url)
        .send()
        .await?
        .error_for_status()?;

    let tar_gz_path = dir.join(format!("{pmcid}.tar.gz"));
    let mut file = File::create(&tar_gz_path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    drop(file);

    tracing::debug!("Extracting OA package for `{pmcid}`");
    let tar_gz = std::fs::File::open(&tar_gz_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&dir)?;

    Ok(dir.join(pmcid))
}

/// Reads any image files in the package and "inlines" them into the node's
/// `content_url` as a dataURI
struct ImageInliner {
    dir: PathBuf,
}

impl ImageInliner {
    fn inline_image(&self, image: &mut ImageObject) {
        for ext in ["", ".png", ".jpg", ".jpeg", ".gif"] {
            let path = self.dir.join(&[&image.content_url, ext].concat());
            if path.exists() {
                if let Ok(url) = images::path_to_data_uri(&path) {
                    image.content_url = url;
                    break;
                }
            }
        }
    }
}

impl VisitorMut for ImageInliner {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::ImageObject(image) = block {
            self.inline_image(image)
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::ImageObject(image) = inline {
            self.inline_image(image)
        }

        WalkControl::Continue
    }
}
