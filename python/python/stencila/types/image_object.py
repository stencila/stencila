# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
from .inline import Inline
from .media_object import MediaObject


@dataclass(kw_only=True, frozen=True)
class ImageObject(MediaObject):
    """
    An image file.
    """

    type: Literal["ImageObject"] = field(default="ImageObject", init=False)

    caption: Optional[List[Inline]] = None
    """The caption for this image."""

    thumbnail: Optional[ImageObject] = None
    """Thumbnail image of this image."""
