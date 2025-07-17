use codec::{
    common::{
        eyre::Result,
        futures::StreamExt,
        reqwest::Response,
        tempfile::tempdir,
        tokio::{fs::File, io::AsyncWriteExt},
        tracing,
    },
    schema::Node,
    Codec, DecodeInfo, DecodeOptions,
};
use codec_pdf::PdfCodec;

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

    let (node, .., info) = PdfCodec.from_path(&temp_file, options).await?;
    Ok((node, info))
}
