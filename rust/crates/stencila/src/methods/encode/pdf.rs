use super::Options;
use chromiumoxide::{cdp::browser_protocol::page::PrintToPdfParamsBuilder, Browser, BrowserConfig};
use codec_html::HtmlCodec;
use codec_trait::{Codec, EncodeOptions};
use eyre::{bail, Result};
use futures::StreamExt;
use stencila_schema::Node;

/// Encode a `Node` to a PDF document
pub async fn encode(node: &Node, output: &str, options: Option<Options>) -> Result<String> {
    let path = if let Some(path) = output.strip_prefix("file://") {
        path
    } else {
        bail!("Can only encode to a file:// output")
    };
    let Options { theme, .. } = options.unwrap_or_default();

    let html = HtmlCodec::to_string(
        node,
        Some(EncodeOptions {
            standalone: true,
            bundle: true,
            theme,
            ..Default::default()
        }),
    )?;

    let chrome = binaries::require("chrome", "*").await?;

    let config = BrowserConfig::builder()
        .chrome_executable(chrome.path)
        .build()
        .expect("Should build config");

    let (browser, mut handler) = Browser::launch(config).await?;
    tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let params = PrintToPdfParamsBuilder::default().build();
    browser
        .new_page("about:blank")
        .await?
        .set_content(html)
        .await?
        .save_pdf(params, path)
        .await?;

    Ok(["file://", path].concat())
}
