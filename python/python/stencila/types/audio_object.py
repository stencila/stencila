# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .media_object import MediaObject


@dataclass(kw_only=True, frozen=True)
class AudioObject(MediaObject):
    """
    An audio file.
    """

    type: Literal["AudioObject"] = field(default="AudioObject", init=False)

    caption: Optional[str] = None
    """The caption for this audio recording."""

    transcript: Optional[str] = None
    """The transcript of this audio recording."""
