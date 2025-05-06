use std::path::PathBuf;

use common::{
    eyre::{Result, bail},
    itertools::Itertools,
    tracing,
};
use document::{DecodeOptions, Document, ExecuteOptions, codecs};
use format::Format;
use schema::Node;

use crate::latex;

/// Knit a document
#[tracing::instrument]
pub async fn knit(
    input: PathBuf,
    outputs: Vec<String>,
    from: Option<Format>,
    to: Option<Format>,
) -> Result<()> {
    let from = from.unwrap_or_else(|| Format::from_path(&input));

    // Decode article from the input file
    let node = codecs::from_path(
        &input,
        Some(DecodeOptions {
            coarse: Some(true),
            ..Default::default()
        }),
    )
    .await?;

    // Execute the article
    let document = Document::from(node, Some(input.to_path_buf())).await?;
    document.execute(ExecuteOptions::default()).await?;
    document.diagnostics_print().await?;

    let Node::Article(article) = document.root().await else {
        bail!("Expected article")
    };

    let mut outputs = outputs.into_iter();
    loop {
        // Get the output file path
        let Some(output) = outputs.next() else { break };
        let output = PathBuf::from(output);

        // Get any passthrough args by collecting any following
        // args that begin with a hyphen.
        let passthrough_args: Vec<String> = outputs
            .take_while_ref(|option| option.starts_with("-"))
            .collect();

        let to = to.clone().unwrap_or_else(|| Format::from_path(&output));

        // Encode to format, with passthrough args
        match from {
            Format::Latex | Format::Tex => match to {
                Format::Latex | Format::Tex => latex::article_to_latex(&article, &output)?,
                Format::Pdf => latex::article_to_pdf(&article, &output, &passthrough_args)?,
                Format::Docx => latex::article_to_docx(&article, &output, &passthrough_args)?,
                _ => bail!("Unsupported to format: {to}"),
            },
            _ => bail!("Unsupported from format: {from}"),
        };
    }

    Ok(())
}
