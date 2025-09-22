use std::path::Path;

use stencila_codec::{Codec, DecodeInfo, DecodeOptions, eyre::Result, stencila_schema::Node};
use stencila_codec_pdf::PdfCodec;

use super::decode::arxiv_id_to_doi;

/// Decode an arXiv PDF file to a Stencila [`Node`]
#[tracing::instrument(skip(options))]
pub(super) async fn decode_arxiv_pdf(
    arxiv_id: &str,
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let (mut node, .., info) = PdfCodec.from_path(path, options).await?;

    // Set doi, and other metadata
    if let Node::Article(article) = &mut node {
        article.doi = Some(arxiv_id_to_doi(arxiv_id));
        article.options.repository = Some("https://arxiv.org".into());
        article.options.path = Some(["pdf/", arxiv_id].concat());
    }

    Ok((node, info))
}
