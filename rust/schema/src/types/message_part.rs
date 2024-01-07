// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::audio_object::AudioObject;
use super::image_object::ImageObject;
use super::string::String;
use super::video_object::VideoObject;

/// A union type for a part of a message.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault)]
#[serde(untagged, crate = "common::serde")]
pub enum MessagePart {
    #[default]
    String(String),

    ImageObject(ImageObject),

    AudioObject(AudioObject),

    VideoObject(VideoObject),
}
