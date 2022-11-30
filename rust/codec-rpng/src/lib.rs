use codec::{
    common::{
        async_trait::async_trait,
        base64,
        eyre::{bail, Result},
    },
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_json::JsonCodec;
use png::{PixelDimensions, Unit};
use std::{fs, path::Path};

/// Encode and decode a document node to a reproducible PNG image.
///
/// This codec encodes the JSON representation of the node in a iTXt chunk of a PNG
/// image. You can use the tool `pnginfo` to examine the text chunks in the
/// encoded image e.g.
///
/// ```sh
/// $ pnginfo my.png
/// ```
///
/// For reference, there is a previous implementation of this codec, written
/// in Node.js at https://github.com/stencila/encoda/blob/v0.119.0/src/codecs/rpng/index.ts.
pub struct RpngCodec {}

#[async_trait]
impl CodecTrait for RpngCodec {
    fn spec() -> Codec {
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["rpng"],
            root_types: vec_string!["*"],
            ..Default::default()
        }
    }

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
    async fn from_path(path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
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
    async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
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
                chunk
                    .decompress_text()
                    .ok()
                    .and_then(|_| chunk.get_text().ok())
            }
            false => None,
        });

    let json = match json {
        Some(json) => json,
        None => bail!("The PNG does not have an embedded Stencila node"),
    };

    let mut node = codec_json::JsonCodec::from_str(&json, options)?;

    // If the node is a `CodeChunk` with one output that is an image, we need to replace
    // `data:self` dataURIs with the image. But first crop it to remove the indicator and
    // any padding that was added during screen-shotting
    if let Node::CodeChunk(chunk) = &mut node {
        if let Some(outputs) = &mut chunk.outputs {
            for output in outputs {
                if let Node::ImageObject(image) = output {
                    if image.content_url == "data:self" {
                        use image::{io::Reader, ImageFormat};
                        use std::io::Cursor;

                        let rpng =
                            Reader::with_format(Cursor::new(bytes), ImageFormat::Png).decode()?;

                        // The padding pixels must be the values for `CodeChunk` outputs in `stencila/themes/src/themes/rpng/styles.css`
                        const PADDING_TOP: u32 = 21;
                        // This adjustment is necessary to avoid the image growing in height after several iterations.
                        // It is not clear where this extra space is coming from and this value was obtained
                        // through experimentation.
                        const ADJUST_HEIGHT: u32 = 4;
                        let rpng = rpng.crop_imm(
                            0,
                            PADDING_TOP,
                            rpng.width(),
                            rpng.height() - PADDING_TOP - ADJUST_HEIGHT,
                        );

                        let mut bytes: Vec<u8> = Vec::new();
                        rpng.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
                        let b64 = base64::encode(&bytes);

                        image.content_url = ["data:image/png;base64,", &b64].concat()
                    }
                }
            }
        }
    }

    Ok(node)
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
            theme: Some("rpng".to_string()),
            ..options.unwrap_or_default()
        }),
    )
    .await?;

    // Transform each PNG into an RPNG...
    let mut rpngs = Vec::with_capacity(nodes.len());
    for index in 0..nodes.len() {
        let mut node = nodes[index].clone();

        // If the node is a `CodeChunk` and it has only one `ImageObject` in its outputs
        // then replace its `content_url` with a special dataURI with refer to itself. This avoid
        // "doubling up" the image data.
        let mut is_code_chunk = false;
        if let Node::CodeChunk(chunk) = &mut node {
            is_code_chunk = true;
            if let Some(outputs) = &mut chunk.outputs {
                if outputs.len() == 1 {
                    if let Node::ImageObject(image) = &mut outputs[0] {
                        image.content_url = "data:self".to_string();
                    }
                }
            }
        }

        // Encode the node to JSON for embedding in the PNG
        let json = JsonCodec::to_string(
            &node,
            Some(EncodeOptions {
                compact: true,
                ..Default::default()
            }),
        )?;

        // Decode the image to get its size etc
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

        // Add a chunk to describe the physical size of the image
        // This is necessary because, for higher resolution images, we scale by two when
        // taking screenshots in the PNG codec. Unless specified, Pandoc assumes
        // images are 96dpi and thus they appear twice as big in DOCX and other formats.
        // Note that for code chunks the RPNG theme scales the node to 50% so the effective
        // scaling is 1.0 (this is done to be able to accurately crop out the image from the
        // RPNG image and to avoid very large images).
        // Implementation based on https://github.com/image-rs/image-png/pull/124/files
        // 96 pixels-per-inch is converted to pixels per meter
        let ppm = (96.0 / 0.0254 * if is_code_chunk { 1.0 } else { 2.0 }) as u32;
        let pixel_dims = PixelDimensions {
            xppu: ppm,
            yppu: ppm,
            unit: Unit::Meter,
        };
        let mut phys = [pixel_dims.xppu.to_be_bytes(), pixel_dims.yppu.to_be_bytes()].concat();
        phys.push(pixel_dims.unit as u8);
        writer.write_chunk(png::chunk::pHYs, &phys)?;

        writer.write_image_data(&image_data)?;
        drop(writer);

        rpngs.push(image_bytes);
    }

    Ok(rpngs)
}

#[cfg(test)]
mod tests {
    use codec::stencila_schema::CodeChunk;
    use test_utils::{
        assert_json_eq,
        common::{tempfile, tokio},
    };

    use super::*;

    /// End-to-end test of encoding a node to a PNG and then decoding
    /// it from the PNG. See `../tests/prop.rs` for more intensive end-to-end testing.
    #[ignore]
    #[tokio::test]
    async fn encode_decode() -> Result<()> {
        let input = Node::CodeChunk(CodeChunk {
            programming_language: "python".to_string(),
            code: "print(\"Hello world!\")".to_string(),
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
