use codec::{
    eyre::{bail, Result},
    stencila_schema::{Article, AudioObject, ImageObject, Node, SoftwareSourceCode, VideoObject},
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions,
};
use formats::{FormatNodeType, FORMATS};

/// A fallback codec that decodes a node based on the format name provided
pub struct FormatCodec {}

impl CodecTrait for FormatCodec {
    fn spec() -> Codec {
        Codec {
            formats: vec_string!["*"],
            root_types: vec_string![
                "Article",
                "AudioObject",
                "ImageObject",
                "VideoObject",
                "SoftwareSourceCode"
            ],
            from_string: true,
            from_path: true,
            ..Default::default()
        }
    }

    fn from_str(content: &str, options: Option<DecodeOptions>) -> Result<Node> {
        let format_name = match options.unwrap_or_default().format {
            Some(format) => format,
            None => bail!("Must provide a format to be decoded"),
        };

        let format = FORMATS.match_name(&format_name);

        let node = match format.node_type {
            FormatNodeType::Article => Node::Article(Article {
                text: Some(Box::new(content.to_string())),
                ..Default::default()
            }),
            FormatNodeType::AudioObject => Node::AudioObject(AudioObject {
                content_url: content.to_string(),
                ..Default::default()
            }),
            FormatNodeType::ImageObject => Node::ImageObject(ImageObject {
                content_url: content.to_string(),
                ..Default::default()
            }),
            FormatNodeType::VideoObject => Node::VideoObject(VideoObject {
                content_url: content.to_string(),
                ..Default::default()
            }),
            FormatNodeType::SoftwareSourceCode => Node::SoftwareSourceCode(SoftwareSourceCode {
                text: Some(Box::new(content.to_string())),
                programming_language: match format_name.is_empty() {
                    true => None,
                    false => Some(Box::new(format_name)),
                },
                ..Default::default()
            }),
            FormatNodeType::Unknown => bail!("Unknown format kind"),
        };
        Ok(node)
    }
}
