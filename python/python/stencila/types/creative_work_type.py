# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Article = ForwardRef("Article")
AudioObject = ForwardRef("AudioObject")
Claim = ForwardRef("Claim")
Collection = ForwardRef("Collection")
Comment = ForwardRef("Comment")
Datatable = ForwardRef("Datatable")
Directory = ForwardRef("Directory")
Figure = ForwardRef("Figure")
File = ForwardRef("File")
ImageObject = ForwardRef("ImageObject")
MediaObject = ForwardRef("MediaObject")
Periodical = ForwardRef("Periodical")
PublicationIssue = ForwardRef("PublicationIssue")
PublicationVolume = ForwardRef("PublicationVolume")
Review = ForwardRef("Review")
SoftwareApplication = ForwardRef("SoftwareApplication")
SoftwareSourceCode = ForwardRef("SoftwareSourceCode")
Table = ForwardRef("Table")
VideoObject = ForwardRef("VideoObject")


CreativeWorkType = Union[
    Article,
    AudioObject,
    Claim,
    Collection,
    Comment,
    Datatable,
    Directory,
    Figure,
    File,
    ImageObject,
    MediaObject,
    Periodical,
    PublicationIssue,
    PublicationVolume,
    Review,
    SoftwareApplication,
    SoftwareSourceCode,
    Table,
    VideoObject,
]
"""
Union type for all types that are descended from `CreativeWork`
"""
