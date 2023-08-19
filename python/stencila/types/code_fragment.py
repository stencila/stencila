# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class CodeFragment(BaseModel):
    """
    Inline code.
    """

    id: Optional[str]
    """The identifier for this item"""

    code: str
    """The code."""

    programming_language: Optional[str]
    """The programming language of the code."""

    media_type: Optional[str]
    """Media type, typically expressed using a MIME format, of the code."""
