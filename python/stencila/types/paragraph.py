# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline


class Paragraph(BaseModel):
    """
    Paragraph
    """

    id: Optional[str]
    """The identifier for this item"""

    content: List[Inline]
    """The contents of the paragraph."""
