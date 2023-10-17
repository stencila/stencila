# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .inline import Inline


@dataclass(kw_only=True, frozen=True)
class Paragraph(Entity):
    """
    A paragraph.
    """

    type: Literal["Paragraph"] = field(default="Paragraph", init=False)

    content: List[Inline]
    """The contents of the paragraph."""
