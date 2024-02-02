# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity


@dataclass(init=False)
class File(Entity):
    """
    A file on the file system.
    """

    type: Literal["File"] = field(default="File", init=False)

    name: str
    """The name of the file."""

    path: str
    """The path (absolute or relative) of the file on the file system"""

    media_type: Optional[str] = None
    """IANA media type (MIME type)."""

    def __init__(self, name: str, path: str, id: Optional[str] = None, media_type: Optional[str] = None):
        super().__init__(id = id)
        self.name = name
        self.path = path
        self.media_type = media_type
