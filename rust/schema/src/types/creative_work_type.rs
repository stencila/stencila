// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The kind of a creative work.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum CreativeWorkType {
    #[default]
    Article,

    AudioObject,

    Blog,

    Book,

    Chapter,

    Chat,

    Claim,

    Collection,

    Comment,

    Dataset,

    Datatable,

    Drawing,

    Figure,

    File,

    ImageObject,

    Legislation,

    Manuscript,

    Map,

    MediaObject,

    Periodical,

    Photograph,

    Poster,

    Presentation,

    Prompt,

    PublicationIssue,

    PublicationVolume,

    Report,

    Review,

    SoftwareApplication,

    SoftwareRepository,

    SoftwareSourceCode,

    Table,

    Thesis,

    VideoObject,

    WebPage,

    Workflow,
}
