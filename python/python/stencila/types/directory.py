# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .collection import Collection
from .file_or_directory import FileOrDirectory


@dataclass(kw_only=True, frozen=True)
class Directory(Collection):
    """
    A directory on the file system.
    """

    type: Literal["Directory"] = field(default="Directory", init=False)

    parts: List[FileOrDirectory]
    """The files and other directories that are within this directory"""

    path: str
    """The path (absolute or relative) of the file on the filesystem"""
