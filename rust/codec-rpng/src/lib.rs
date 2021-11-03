use codec_json::JsonCodec;
use codec_trait::{
    async_trait::async_trait,
    eyre::{bail, Result},
    stencila_schema::Node,
    Codec, DecodeOptions, EncodeOptions,
};
use std::{fs, path::Path};

/// Encode and decode a document node to a reproducible PNG image.
///
/// This codec encodes the JSON representation of the node in a iTXt chunk of a PNG
/// image. You can use the tool `pnginfo` to examine the text chunks in the
/// encoded image e.g.
///
/// ```sh
/// pnginfo my.png
/// ```
///
/// For reference, there is a previous implementation of this codec, written
/// in Node.js at https://github.com/stencila/encoda/blob/v0.119.0/src/codecs/rpng/index.ts.
pub struct RpngCodec {}

#[async_trait]
impl Codec for RpngCodec {
    /// Decode a document node from a string
    ///
    /// This function scans the PNG for a `iTXt` chunk with a matching keyword and then delegates
    /// to the [`JsonCodec`] to decode it.
    fn from_str(content: &str, options: Option<DecodeOptions>) -> Result<Node> {
        // Remove any dataURI prefix
        let data = if let Some(data) = content.strip_prefix("data:image/png;base64,") {
            data
        } else {
            content
        };

        // Decode the Base64 to bytes and then a node
        let bytes = base64::decode(data)?;
        bytes_to_node(bytes.as_slice(), options)
    }

    /// Decode a document node from a file system path
    ///
    /// This override is necessary to read the file as bytes, not as a string, and then to
    /// directly decode those bytes, rather than Base64 decoding them first.
    /// Decode a document node from a file system path
    async fn from_path<T: AsRef<Path>>(path: &T, options: Option<DecodeOptions>) -> Result<Node>
    where
        T: Send + Sync,
    {
        let bytes = fs::read(path)?;
        bytes_to_node(bytes.as_slice(), options)
    }

    /// Encode a document node to a string
    ///
    /// Returns a Base64 encoded dataURI (with media type `image/png`) and the node embedded as JSON.
    async fn to_string_async(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        let bytes = nodes_to_bytes(&[node], options).await?;
        let string = ["data:image/png;base64,", &base64::encode(&bytes[0])].concat();
        Ok(string)
    }

    /// Encode a document node to a file system path
    ///
    /// This override is necessary to avoid the dataURI prefix and Base64 encoding that `to_string_async`
    /// does. It simply writes that bytes to a file at the path.
    async fn to_path<T: AsRef<Path>>(
        node: &Node,
        path: &T,
        options: Option<EncodeOptions>,
    ) -> Result<()>
    where
        T: Send + Sync,
    {
        let bytes = nodes_to_bytes(&[node], options).await?;
        fs::write(path, &bytes[0])?;
        Ok(())
    }
}

/// Decode bytes to a document node
fn bytes_to_node(bytes: &[u8], options: Option<DecodeOptions>) -> Result<Node> {
    // Decode the bytes to PNG and extract any matching iTXt chunk
    let decoder = png::Decoder::new(bytes);
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
        Some(json) => codec_json::JsonCodec::from_str(&json, options),
        None => bail!("The PNG does not have an embedded Stencila node"),
    }
}

/// Encode a set of document nodes to RPNGs as bytes
///
/// As with the `nodes_to_bytes` function in the `codec-png` crate, this function
/// is based around creating multiple RPNGs, rather than a single one, to
/// reduce the per-image overhead of starting the browser, loading the theme etc.
pub async fn nodes_to_bytes(
    nodes: &[&Node],
    options: Option<EncodeOptions>,
) -> Result<Vec<Vec<u8>>> {
    // Generate the plain old PNGs
    let pngs = codec_png::nodes_to_bytes(
        nodes,
        Some(EncodeOptions {
            theme: "rpng".to_string(),
            ..options.unwrap_or_default()
        }),
    )
    .await?;

    // Transform each PNG into an RPNG...
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
    use super::*;
    use codec_trait::stencila_schema::CodeChunk;
    use test_utils::assert_json_eq;

    /// End-to-end test of encoding a node to a PNG and then decoding
    /// it from the PNG. See `../tests/prop.rs` for more intensive end-to-end testing.
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn encode_decode() -> Result<()> {
        let input = Node::CodeChunk(CodeChunk {
            programming_language: "python".to_string(),
            text: "print(\"Hello world!\")".to_string(),
            outputs: Some(vec![Node::String("Hello world!".to_string())]),
            ..Default::default()
        });

        let data_uri = RpngCodec::to_string_async(&input, None).await?;
        assert!(data_uri.starts_with("data:image/png;base64,"));
        let output = RpngCodec::from_str(&data_uri, None)?;
        assert_json_eq!(input, output);

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("temp.png");
        RpngCodec::to_path(&input, &path, None).await?;
        assert!(path.exists());
        let output = RpngCodec::from_path(&path, None).await?;
        assert_json_eq!(input, output);

        Ok(())
    }
}
