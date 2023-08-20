# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork


@dataclass(kw_only=True, frozen=True)
class MediaObject(CreativeWork):
    """
    A media object, such as an image, video, or audio object embedded in a web page or a    downloadable dataset.
    """

    type: Literal["MediaObject"] = field(default="MediaObject", init=False)

    bitrate: Optional[float] = None
    """Bitrate in megabits per second (Mbit/s, Mb/s, Mbps)."""

    content_size: Optional[float] = None
    """File size in megabits (Mbit, Mb)."""

    content_url: str
    """URL for the actual bytes of the media object, for example the image file or video file."""

    embed_url: Optional[str] = None
    """URL that can be used to embed the media on a web page via a specific media player."""

    media_type: Optional[str] = None
    """IANA media type (MIME type)."""
