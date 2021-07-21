use super::html;
use crate::utils::browser;
use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParamsBuilder;
use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to a PDF document
pub async fn encode(node: &Node, output: &str, theme: &str) -> Result<String> {
    let path = if let Some(path) = output.strip_prefix("file://") {
        path
    } else {
        output
    };

    let html = html::encode(node, true, theme)?;

    let page = browser::page().await?;
    let params = PrintToPdfParamsBuilder::default().build();
    page.set_content(html).await?.save_pdf(params, path).await?;

    Ok(["file://", path].concat())
}
