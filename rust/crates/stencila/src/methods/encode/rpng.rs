use super::{
    png::{encode_to_output, encode_to_pngs},
    Options,
};
use codec_json::JsonCodec;
use codec_trait::{Codec, EncodeOptions};
use eyre::Result;
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
pub async fn encode(node: &Node, output: &str) -> Result<String> {
    let pngs = encode_to_rpngs(&[node]).await?;
    encode_to_output(&pngs[0], output)
}

/// Encode a list of `Node`s to RPNGs (as bytes)
pub async fn encode_to_rpngs(nodes: &[&Node]) -> Result<Vec<Vec<u8>>> {
    let pngs = encode_to_pngs(
        nodes,
        Some(Options {
            theme: "rpng".to_string(),
            ..Default::default()
        }),
    )
    .await?;
    let mut rpngs = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        // Encode the node to JSON for embedding in the PNG
        let json = JsonCodec::to_string(
            nodes[index],
            Some(EncodeOptions {
                theme: "compact".to_string(),
                ..Default::default()
            }),
        )?;

        // Encode the node to a PNG, as bytes, and decode the image to get its size etc
        let image_bytes = &pngs[index];
        let decoder = png::Decoder::new(image_bytes.as_slice());
        let mut reader = decoder.read_info()?;
        let mut image_data = vec![0; reader.output_buffer_size()];
        let image_info = reader.next_frame(&mut image_data)?;

        // Re-encode the PNG, as bytes, with JSON embedded
        let mut image_bytes: Vec<u8> = Vec::new();
        let mut encoder = png::Encoder::new(&mut image_bytes, image_info.width, image_info.height);
        encoder.set_color(image_info.color_type);
        encoder.set_depth(image_info.bit_depth);
        encoder.add_itxt_chunk("json".to_string(), json).unwrap();
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&image_data)?;
        drop(writer);

        rpngs.push(image_bytes);
    }
    Ok(rpngs)
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
        let output = encode(&node, &path.to_slash_lossy()).await?;
        assert_eq!(output, ["file://", &path.to_slash_lossy()].concat());
        assert!(path.exists());

        let data = encode(&node, "data://").await?;
        assert!(data.starts_with("data:image/png;base64,"));

        Ok(())
    }
}
