use std::path::PathBuf;

use common::{
    eyre::{Result, bail},
    itertools::Itertools,
    tracing,
};
use document::{Document, ExecuteOptions};
use format::Format;
use schema::Node;

use crate::latex;

/// Knit a document
#[tracing::instrument]
pub(super) async fn knit(
    input: PathBuf,
    outputs: Vec<String>,
    from: Option<Format>,
    to: Option<Format>,
) -> Result<()> {
    let from = from.unwrap_or_else(|| Format::from_path(&input));

    // Decode from the input file
    let article = match from {
        Format::Latex | Format::Tex => latex::latex_to_article(&input),
        _ => bail!("Unsupported from format: {from}"),
    }?;

    // Execute the article
    let document = Document::from(Node::Article(article), Some(input.to_path_buf())).await?;
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
