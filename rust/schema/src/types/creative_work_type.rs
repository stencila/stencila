// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::article::Article;
use super::audio_object::AudioObject;
use super::claim::Claim;
use super::collection::Collection;
use super::comment::Comment;
use super::datatable::Datatable;
use super::figure::Figure;
use super::image_object::ImageObject;
use super::media_object::MediaObject;
use super::periodical::Periodical;
use super::publication_issue::PublicationIssue;
use super::publication_volume::PublicationVolume;
use super::review::Review;
use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::table::Table;
use super::video_object::VideoObject;

/// Union type for all types that are descended from `CreativeWork`
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum CreativeWorkType {
    #[default]
    Article(Article),

    AudioObject(AudioObject),

    Claim(Claim),

    Collection(Collection),

    Comment(Comment),

    Datatable(Datatable),

    Figure(Figure),

    ImageObject(ImageObject),

    MediaObject(MediaObject),

    Periodical(Periodical),

    PublicationIssue(PublicationIssue),

    PublicationVolume(PublicationVolume),

    Review(Review),

    SoftwareApplication(SoftwareApplication),

    SoftwareSourceCode(SoftwareSourceCode),

    Table(Table),

    VideoObject(VideoObject),
}
