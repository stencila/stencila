use std::path::Path;

use glob::glob;
use stencila_node_supplements::embed_supplements;
use zip::ZipArchive;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions,
    eyre::{OptionExt, Result, WrapErr},
    stencila_schema::Node,
};
use stencila_codec_jats::JatsCodec;
use stencila_node_media::{embed_media, relocate_media, side_dir};

/// Decode a MECA file to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Create temporary directory to extract into
    // if path is not already a directory (e.g. an unzipped MECA)
    let tempdir = tempfile::TempDir::new()?;
    let dir = if path.is_dir() { path } else { tempdir.path() };

    if path.is_file() {
        tracing::debug!("Extracting MECA");
        let file = std::fs::File::open(path)?;
        let mut zip = ZipArchive::new(file)?;
        zip.extract(dir)?;
    }

    let dir = dir.join("content");

    // Find the JATS file in the dir
    let jats_path = glob(&dir.join("*.xml").to_string_lossy())?
        .next()
        .and_then(|res| res.ok())
        .ok_or_eyre("Unable to find JATS XML file")?;

    // Decode the JATS
    let options = options.unwrap_or_default();
    let embed_images = options.embed_media.unwrap_or(true);
    let embed_works = options.embed_supplements.unwrap_or(true);
    let (mut node, .., info) = JatsCodec
        .from_path(
            &jats_path,
            Some(DecodeOptions {
                // Apply archive defaults here, once, after resolving JATS hrefs.
                embed_media: Some(false),
                embed_supplements: Some(false),
                ..options
            }),
        )
        .await?;

    // Embed media and supplements
    if embed_images {
        embed_media(&mut node, Some(&dir))?;
    }
    if embed_works {
        embed_supplements(&mut node, &dir).await?;
    }

    // References must outlive the temporary extraction directory. Retain only
    // referenced originals in side directories beside the archive; do not
    // embed and then extract, which would resize/transcode the originals.
    let media_dir = (!embed_images && path.is_file()).then(|| side_dir(path, ".media"));
    let supplements_dir = (!embed_works && path.is_file()).then(|| side_dir(path, ".supplements"));
    if !embed_images || !embed_works {
        relocate_media(
            &mut node,
            Some(&jats_path),
            Some(path),
            media_dir.as_deref(),
            supplements_dir.as_deref(),
        )
        .wrap_err_with(|| retain_error(path))?;
    }

    Ok((node, None, info))
}

/// Error context when originals can not be retained beside an archive
fn retain_error(path: &Path) -> String {
    format!(
        "Could not retain original media and supplements beside `{}`; is its directory writable? \
         Decode with embedding enabled (e.g. `--embed-media --embed-supplements`) or decode a \
         writable copy of the archive",
        path.display()
    )
}
