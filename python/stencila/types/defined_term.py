# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .image_object_or_str import ImageObjectOrStr
from .property_value_or_str import PropertyValueOrStr


class DefinedTerm(BaseModel):
    """
    A word, name, acronym, phrase, etc. with a formal definition.
    """

    id: Optional[str]
    """The identifier for this item"""

    alternate_names: Optional[List[str]]
    """Alternate names (aliases) for the item."""

    description: Optional[List[Block]]
    """A description of the item."""

    identifiers: Optional[List[PropertyValueOrStr]]
    """Any kind of identifier for any kind of Thing."""

    images: Optional[List[ImageObjectOrStr]]
    """Images of the item."""

    name: str
    """The name of the item."""

    url: Optional[str]
    """The URL of the item."""

    term_code: Optional[str]
    """A code that identifies this DefinedTerm within a DefinedTermSet"""
