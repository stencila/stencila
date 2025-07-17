use flate2::read::GzDecoder;
use std::io::Read;

use codec::{
    common::{
        eyre::{bail, Context, Result},
        reqwest::Response,
        tar, tracing,
    },
    schema::Node,
    Codec, DecodeInfo, DecodeOptions,
};
use codec_latex::LatexCodec;

/// Decode the response from an arXiv `src` URL to a Stencila [`Node`]
#[tracing::instrument(skip(options, response))]
pub(super) async fn decode_arxiv_src(
    arxiv_id: &str,
    response: Response,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let bytes = response.bytes().await?;

    let mut decoder = GzDecoder::new(bytes.as_ref());
    let mut decompressed = Vec::new();

    // arXiv serves either gzipped single files or gzipped tar files
    // Try to decompress with gzip first
    let latex = if decoder.read_to_end(&mut decompressed).is_ok() {
        // Check if it's a tar file by looking for tar magic bytes
        if decompressed.len() > 262 && &decompressed[257..262] == b"ustar" {
            tracing::trace!("Detected tar archive, extracting .tex files");

            // Extract .tex files from tar
            let mut archive = tar::Archive::new(decompressed.as_slice());
            let mut latex_content = String::new();

            for entry in archive.entries().wrap_err("Failed to read tar entries")? {
                let mut entry = entry.wrap_err("Failed to read tar entry")?;

                if let Ok(path) = entry.path() {
                    if let Some(extension) = path.extension() {
                        if extension == "tex" {
                            tracing::trace!("Found .tex file: {:?}", path);
                            entry
                                .read_to_string(&mut latex_content)
                                .wrap_err("Failed to read .tex file")?;
                            break;
                        }
                    }
                }
            }

            if latex_content.is_empty() {
                bail!("No .tex file found in arXiv source archive");
            }

            latex_content
        } else {
            // Single decompressed file, assume it's LaTeX
            tracing::trace!("Single file detected, treating as LaTeX");
            String::from_utf8(decompressed).wrap_err("Failed to decode LaTeX as UTF-8")?
        }
    } else {
        // If gzip decompression fails, try treating as plain text
        tracing::debug!("Gzip decompression failed, trying as plain text");
        String::from_utf8(bytes.to_vec()).wrap_err("Failed to decode response as UTF-8")?
    };

    if latex.trim().is_empty() {
        bail!("Retrieved LaTeX content is empty")
    }

    LatexCodec
        .from_str(
            &latex,
            Some(DecodeOptions {
                coarse: Some(false),
                ..options.unwrap_or_default()
            }),
        )
        .await
}
