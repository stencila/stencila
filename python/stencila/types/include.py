# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .executable import Executable


@dataclass(kw_only=True, frozen=True)
class Include(Executable):
    """
    Include content from an external source (e.g. file, URL).
    """

    type: Literal["Include"] = field(default="Include", init=False)

    source: str
    """The external source of the content, a file path or URL."""

    media_type: Optional[str] = None
    """Media type of the source content."""

    select: Optional[str] = None
    """A query to select a subset of content from the source"""

    content: Optional[List[Block]] = None
    """The structured content decoded from the source."""
