use crate::formats::FormatType;
use eyre::Result;
use stencila_schema::{AudioObject, ImageObject, Node, VideoObject};

/// Decode input to a type of `MediaObject` having the input as its `content_url`.
pub fn decode(input: &str, format: FormatType) -> Result<Node> {
    Ok(match format {
        FormatType::AudioObject => Node::AudioObject(AudioObject {
            content_url: input.to_string(),
            ..Default::default()
        }),
        FormatType::ImageObject => Node::ImageObject(ImageObject {
            content_url: input.to_string(),
            ..Default::default()
        }),
        FormatType::VideoObject => Node::VideoObject(VideoObject {
            content_url: input.to_string(),
            ..Default::default()
        }),
        _ => unreachable!()
    })
}
