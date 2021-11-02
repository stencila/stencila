use chromiumoxide::{cdp::browser_protocol::page::PrintToPdfParamsBuilder, Browser, BrowserConfig};
use codec_html::HtmlCodec;
use codec_trait::{
    async_trait::async_trait, eyre::Result, stencila_schema::Node, Codec, EncodeOptions,
};
use futures::StreamExt;
use std::path::Path;

/// A codec for PDF files
///
/// This codec uses a headless browser to take a screenshot of the HTML
/// encoding of a document.
pub struct PdfCodec {}

#[async_trait]
impl Codec for PdfCodec {
    /// Encode a document node to a file system path
    async fn to_path<T: AsRef<Path>>(
        node: &Node,
        path: &T,
        options: Option<EncodeOptions>,
    ) -> Result<()>
    where
        T: Send + Sync,
    {
        let EncodeOptions { theme, .. } = options.unwrap_or_default();

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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec_trait::stencila_schema::Article;

    #[cfg(target_os = "linux")]
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
