# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
from .media_object import MediaObject


@dataclass(kw_only=True, frozen=True)
class VideoObject(MediaObject):
    """
    A video file.
    """

    type: Literal["VideoObject"] = field(default="VideoObject", init=False)

    caption: Optional[str] = None
    """The caption for this video recording."""

    thumbnail: Optional[ImageObject] = None
    """Thumbnail image of this video recording."""

    transcript: Optional[str] = None
    """The transcript of this video recording."""
