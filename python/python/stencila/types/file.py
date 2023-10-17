# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork


@dataclass(kw_only=True, frozen=True)
class File(CreativeWork):
    """
    A file on the file system.
    """

    type: Literal["File"] = field(default="File", init=False)

    path: str
    """The path (absolute or relative) of the file on the filesystem"""
