use std::path::Path;

use common::{eyre::{bail, Result}, tracing};
use document::{Document, ExecuteOptions};
use format::Format;
use schema::Node;

use crate::latex;

/// Knit a document
#[tracing::instrument]
pub(super) async fn knit(
    input: &Path,
    output: &Path,
    from: Option<Format>,
    to: Option<Format>,
) -> Result<()> {
    let from = from.unwrap_or_else(|| Format::from_path(input));
    let to = to.unwrap_or_else(|| Format::from_path(output));

    let article = match from {
        Format::Latex | Format::Tex => latex::latex_to_article(&input),
        _ => bail!("Unsupported from format: {from}"),
    }?;

    let document = Document::from(Node::Article(article), Some(input.to_path_buf())).await?;
    document.execute(ExecuteOptions::default()).await?;
    document.diagnostics_print().await?;

    let Node::Article(article) = document.root().await else {
        bail!("Expected article")
    };

    match from {
        Format::Latex | Format::Tex => match to {
            Format::Latex | Format::Tex => latex::article_to_latex(&article, &output)?,
            Format::Pdf => latex::article_to_pdf(&article, &output)?,
            Format::Docx => latex::article_to_docx(&article, &output)?,
            _ => bail!("Unsupported to format: {to}"),
        },
        _ => bail!("Unsupported from format: {from}"),
    };

    Ok(())
}
