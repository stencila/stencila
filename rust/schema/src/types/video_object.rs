//! Generated file, do not edit

use crate::prelude::*;

use super::image_object::ImageObject;
use super::number::Number;
use super::string::String;

/// A video file.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct VideoObject {
    /// URL for the actual bytes of the media object, for example the image file or video file.
    content_url: String,

    /// IANA media type (MIME type).
    media_type: Option<String>,

    /// The caption for this video recording.
    caption: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<VideoObjectOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct VideoObjectOptions {
    /// Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
    bitrate: Option<Number>,

    /// File size in megabits (Mbit, Mb).
    content_size: Option<Number>,

    /// URL that can be used to embed the media on a web page via a specific media player.
    embed_url: Option<String>,

    /// Thumbnail image of this video recording.
    thumbnail: Option<ImageObject>,

    /// The transcript of this video recording.
    transcript: Option<String>,
}
