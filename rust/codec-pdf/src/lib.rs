use std::path::Path;

use chromiumoxide::{cdp::browser_protocol::page::PrintToPdfParamsBuilder, Browser, BrowserConfig};

use codec::{
    common::{async_trait::async_trait, eyre::Result, futures::StreamExt, tokio},
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, EncodeOptions,
};
use codec_html::HtmlCodec;

/// A codec for PDF files
///
/// This codec uses a headless browser to take a screenshot of the HTML
/// encoding of a document.
pub struct PdfCodec {}

#[async_trait]
impl CodecTrait for PdfCodec {
    fn spec() -> Codec {
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["pdf"],
            root_types: vec_string!["Article"],
            from_string: false,
            from_path: false,
            to_string: false,
            ..Default::default()
        }
    }

    /// Encode a document node to a file system path
    async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
        let EncodeOptions { theme, .. } = options.unwrap_or_default();

        let html = HtmlCodec::to_string(
            node,
            Some(EncodeOptions {
                standalone: true,
                bundle: true,
                theme,
                components: false,
                ..Default::default()
            }),
        )?;

        let chrome = binaries::require_any(&[("chrome", "*"), ("chromium", "*")]).await?;

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

        // Create a page, set its HTML and wait for "navigation"
        let page = browser.new_page("about:blank").await?;
        page.set_content(html).await?.wait_for_navigation().await?;

        // Save as PDF
        let params = PrintToPdfParamsBuilder::default().build();
        page.save_pdf(params, path).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use codec::stencila_schema::Article;
    use test_utils::common::tempfile;

    use super::*;

    #[tokio::test]
    async fn encode() -> super::Result<()> {
        let node = Node::Article(Article::default());

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("temp.pdf");
        PdfCodec::to_path(&node, &path, None).await?;
        assert!(path.exists());

        Ok(())
    }
}
