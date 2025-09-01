use futures::StreamExt;
use reqwest::Response;
use tempfile::tempdir;
use tokio::{fs::File, io::AsyncWriteExt};

use codec::{Codec, DecodeInfo, DecodeOptions, eyre::Result, schema::Node};
use codec_pdf::PdfCodec;

use super::decode::arxiv_id_to_doi;

/// Decode the response from an arXiv `pdf` URL to a Stencila [`Node`]
#[tracing::instrument(skip(options, response))]
pub(super) async fn decode_arxiv_pdf(
    arxiv_id: &str,
    response: Response,
    options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    let temp_dir = tempdir()?;
    let temp_file = temp_dir.path().join(format!("{arxiv_id}.pdf"));

    let mut file = File::create(&temp_file).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    drop(file);

    let (mut node, .., info) = PdfCodec.from_path(&temp_file, options).await?;

    // Set doi, and other metadata
    if let Node::Article(article) = &mut node {
        article.doi = Some(arxiv_id_to_doi(arxiv_id));
        article.options.repository = Some("https://arxiv.org".into());
        article.options.path = Some(["pdf/", arxiv_id].concat());
    }

    Ok((node, info))
}
