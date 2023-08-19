# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .text_value import TextValue


class Text(BaseModel):
    """
    Textual content
    """

    id: Optional[str]
    """The identifier for this item"""

    value: TextValue
    """The value of the text content"""
