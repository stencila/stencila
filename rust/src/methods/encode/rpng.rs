use super::{json, png::encode_to_bytes, Options};
use eyre::Result;
use std::fs;
use stencila_schema::Node;

/// Encode a `Node` to a "Reproducible PNG" image.
///
/// Encodes the JSON representation of the node in the iTXt chunk of the PNG
/// image.
///
/// You can use the tool `pnginfo` to examine the text chunks in the
/// encoded image.
///
/// For reference, there is a previous implementation of this function, written
/// in Node.js at https://github.com/stencila/encoda/blob/v0.119.0/src/codecs/rpng/index.ts.
pub async fn encode(node: &Node, output: &str, options: Option<Options>) -> Result<String> {
    // Encode the node to JSON for embedding in the PNG
    let json = json::encode(
        node,
        Some(Options {
            theme: "compact".to_string(),
            ..Default::default()
        }),
    )?;

    // Encode the node to a PNG, as bytes, and decode the image to get its size etc
    let image_bytes = encode_to_bytes(node, options).await?;
    let decoder = png::Decoder::new(image_bytes.as_slice());
    let mut reader = decoder.read_info()?;
    let mut image_data = vec![0; reader.output_buffer_size()];
    let image_info = reader.next_frame(&mut image_data)?;

    // Re-encode the PNG, as bytes, with JSON embedded
    let mut image_bytes: Vec<u8> = Vec::new();
    let mut encoder = png::Encoder::new(&mut image_bytes, image_info.width, image_info.height);
    encoder.set_color(image_info.color_type);
    encoder.set_depth(image_info.bit_depth);
    encoder.add_itxt_chunk("json", &json).unwrap();
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&image_data)?;
    drop(writer);

    // Encode to a data URI or to file
    let content = if output.starts_with("data:") {
        ["data:image/png;base64,", &base64::encode(&image_bytes)].concat()
    } else {
        let path = if let Some(path) = output.strip_prefix("file://") {
            path
        } else {
            output
        };
        fs::write(path, image_bytes)?;
        ["file://", path].concat()
    };
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_schema::CodeChunk;

    #[tokio::test]
    async fn encode_to_file() -> Result<()> {
        let node = Node::CodeChunk(CodeChunk {
            ..Default::default()
        });
        encode(&node, "file://temp.png", None).await?;
        Ok(())
    }
}
