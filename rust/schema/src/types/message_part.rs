// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::audio_object::AudioObject;
use super::image_object::ImageObject;
use super::text::Text;
use super::video_object::VideoObject;

/// A union type for a part of a message.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum MessagePart {
    #[default]
    Text(Text),

    ImageObject(ImageObject),

    AudioObject(AudioObject),

    VideoObject(VideoObject),
}
