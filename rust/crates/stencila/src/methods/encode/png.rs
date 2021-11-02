use super::Options;
use chromiumoxide::{cdp::browser_protocol::page::CaptureScreenshotFormat, Browser, BrowserConfig};
use codec_html::HtmlCodec;
use codec_trait::{Codec, EncodeOptions};
use eyre::Result;
use futures::StreamExt;
use std::fs;
use stencila_schema::Node;

/// Encode a `Node` to a PNG file or Base64 encoded data URI.
pub async fn encode(node: &Node, output: &str, options: Option<Options>) -> Result<String> {
    let pngs = encode_to_pngs(&[node], options).await?;
    encode_to_output(&pngs[0], output)
}

/// Encode a list of `Node`s to PNGs (as bytes)
pub async fn encode_to_pngs(nodes: &[&Node], options: Option<Options>) -> Result<Vec<Vec<u8>>> {
    // Return early if possible to avoid the following, including requiring Chrome.
    if nodes.is_empty() {
        return Ok(Vec::new());
    }

    // Generate HTML for each node
    let mut html = String::new();
    for (index, node) in nodes.iter().enumerate() {
        let node_html = HtmlCodec::to_string(
            node,
            Some(EncodeOptions {
                standalone: false,
                bundle: true,
                ..Default::default()
            }),
        )?;
        html.push_str(&format!(r#"<div id="node-{}">{}</div>"#, index, node_html));
    }

    // Wrap the HTML with a header etc so that the theme is set and CSS is loaded
    let Options { theme, .. } = options.unwrap_or_default();
    let html = codec_html::wrap_standalone("PNG", &theme, &html);

    // Launch the browser
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

    // Create a page and set its HTML
    let page = browser.new_page("about:blank").await?;
    page.set_content(html).await?.wait_for_navigation().await?;

    // Take a screenshot of each element
    let mut pngs = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        let element = page.find_element(&format!("#node-{}", index)).await?;
        let bytes = element.screenshot(CaptureScreenshotFormat::Png).await?;
        pngs.push(bytes)
    }

    Ok(pngs)
}

/// Encode a PNG (as bytes) to a data URI or a file
pub fn encode_to_output(bytes: &[u8], output: &str) -> Result<String> {
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

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_encode() -> super::Result<()> {
        use super::*;
        use path_slash::PathExt;
        use stencila_schema::CodeChunk;

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
