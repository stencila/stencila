use super::json;
use eyre::{bail, Result};
use std::fs;
use stencila_schema::Node;

/// Decode a "Reproducible PNG" to a `Node`.
///
/// Extracts the JSON representation of the node from any iTXt chunk
/// in the image that has the keyword `json`.
pub fn decode(input: &str) -> Result<Node> {
    // Decode the PNG bytes from the data URI or file
    let image_bytes = if let Some(data) = input.strip_prefix("data:image/png;base64,") {
        base64::decode(data)?
    } else {
        let path = if let Some(path) = input.strip_prefix("file://") {
            path
        } else {
            input
        };
        fs::read(path)?
    };

    // Decode the bytes and check if there is a matching iTXt chunk
    let decoder = png::Decoder::new(image_bytes.as_slice());
    let reader = decoder.read_info()?;
    let info = reader.info();
    let json = info
        .utf8_text
        .iter()
        .find_map(|chunk| match chunk.keyword == "json" {
            true => {
                let mut chunk = chunk.clone();
                if chunk.decompress_text().is_err() {
                    return None;
                }
                chunk.get_text().ok()
            }
            false => None,
        });

    match json {
        Some(json) => json::decode(&json),
        None => bail!("The PNG does not have an embedded Stencila node"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::encode::rpng::encode;
    use stencila_schema::CodeChunk;

    // End-to-end test of encoding a node to a PNG and then decoding
    // it from the PNG. See the integration tests in `ende.rs` for more.
    #[tokio::test]
    async fn encode_decode() -> Result<()> {
        let input = Node::CodeChunk(CodeChunk {
            programming_language: "python".to_string(),
            text: "Some code".to_string(),
            ..Default::default()
        });

        let data_uri = encode(&input, "data://", None).await?;
        let output = decode(&data_uri)?;

        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        );

        Ok(())
    }
}
