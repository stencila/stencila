# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .file_or_directory import FileOrDirectory


@dataclass(init=False)
class Directory(Entity):
    """
    A directory on the file system.
    """

    type: Literal["Directory"] = field(default="Directory", init=False)

    name: str
    """The name of the directory."""

    path: str
    """The path (absolute or relative) of the file on the file system."""

    parts: List[FileOrDirectory]
    """The files and other directories within this directory."""

    def __init__(self, name: str, path: str, parts: List[FileOrDirectory], id: Optional[str] = None):
        super().__init__(id = id)
        self.name = name
        self.path = path
        self.parts = parts
