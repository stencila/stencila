use super::{html, Options};
use crate::binaries;
use chromiumoxide::{Browser, BrowserConfig, cdp::browser_protocol::page::{CaptureScreenshotFormat, CaptureScreenshotParams}};
use eyre::{bail, Result};
use futures::StreamExt;
use stencila_schema::Node;

/// Encode a `Node` to a PNG image
pub async fn encode(node: &Node, output: &str, options: Option<Options>) -> Result<String> {
    let Options { theme, .. } = options.unwrap_or_default();

    let html = html::encode(
        node,
        Some(Options {
            standalone: true,
            bundle: true,
            theme,
        }),
    )?;
    println!("{}", html);

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
    let content = if output.starts_with("data:") {
        let bytes = element.screenshot(format).await?;
        let data_uri = ["data:image/png;base64,", &base64::encode(&bytes)].concat();
        data_uri
    } else {
        let path = if let Some(path) = output.strip_prefix("file://") {
            path
        } else {
            output
        };
        element.save_screenshot(format, path).await?;
        ["file://", &path].concat()
    };

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use stencila_schema::{CodeChunk, CodeExpression};

    #[tokio::test]
    async fn encode_file() -> Result<()> {
        let node = Node::CodeChunk(CodeChunk {
            programming_language: "python".to_string(),
            text: "print(\"Hello world!\")".to_string(),
            outputs: Some(vec![Node::String("Hello world!".to_string())]),
            ..Default::default()
        });
        encode(
            &node,
            "file://temp.png",
            Some(Options {
                theme: "rpng".to_string(),
                ..Default::default()
            }),
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn encode_data_uri() -> Result<()> {
        let node = Node::CodeExpression(CodeExpression {
            ..Default::default()
        });
        encode(&node, "data://", None).await?;
        Ok(())
    }
}
