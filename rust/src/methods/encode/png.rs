use super::{html, Options};
use crate::binaries;
use chromiumoxide::{cdp::browser_protocol::page::CaptureScreenshotFormat, Browser, BrowserConfig};
use eyre::Result;
use futures::StreamExt;
use std::fs;
use stencila_schema::Node;

/// Encode a `Node` to a PNG file or Base64 encoded data URI.
pub async fn encode(node: &Node, output: &str, options: Option<Options>) -> Result<String> {
    let bytes = encode_to_bytes(node, options).await?;
    let content = if output.starts_with("data:") {
        ["data:image/png;base64,", &base64::encode(&bytes)].concat()
    } else {
        let path = if let Some(path) = output.strip_prefix("file://") {
            path
        } else {
            output
        };
        fs::write(path, bytes)?;
        ["file://", path].concat()
    };
    Ok(content)
}

/// Encode a `Node` to the bytes of a PNG image
pub async fn encode_to_bytes(node: &Node, options: Option<Options>) -> Result<Vec<u8>> {
    let Options { theme, .. } = options.unwrap_or_default();

    let html = html::encode(
        node,
        Some(Options {
            standalone: true,
            bundle: true,
            theme,
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

    let page = browser.new_page("about:blank").await?;
    let element = page
        .set_content(html)
        .await?
        .wait_for_navigation()
        .await?
        .find_element("body")
        .await?;

    let format = CaptureScreenshotFormat::Png;
    let bytes = element.screenshot(format).await?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use path_slash::PathExt;
    use stencila_schema::CodeChunk;

    #[tokio::test]
    async fn test_encode() -> Result<()> {
        let node = Node::CodeChunk(CodeChunk {
            programming_language: "python".to_string(),
            text: "print(\"Hello world!\")".to_string(),
            outputs: Some(vec![Node::String("Hello world!".to_string())]),
            ..Default::default()
        });

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("temp.png");
        let output = encode(
            &node,
            &path.to_slash_lossy(),
            Some(Options {
                theme: "rpng".to_string(),
                ..Default::default()
            }),
        )
        .await?;
        assert_eq!(output, ["file://", &path.to_slash_lossy()].concat());
        assert!(path.exists());

        let data = encode(&node, "data://", None).await?;
        assert!(data.starts_with("data:image/png;base64,"));

        Ok(())
    }
}
