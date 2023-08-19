# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .article import Article
from .audio_object import AudioObject
from .claim import Claim
from .collection import Collection
from .comment import Comment
from .datatable import Datatable
from .directory import Directory
from .figure import Figure
from .file import File
from .image_object import ImageObject
from .media_object import MediaObject
from .periodical import Periodical
from .publication_issue import PublicationIssue
from .publication_volume import PublicationVolume
from .review import Review
from .software_application import SoftwareApplication
from .software_source_code import SoftwareSourceCode
from .table import Table
from .video_object import VideoObject


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
