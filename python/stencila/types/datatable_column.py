# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .array_validator import ArrayValidator
from .block import Block
from .image_object_or_str import ImageObjectOrStr
from .primitive import Primitive
from .property_value_or_str import PropertyValueOrStr


class DatatableColumn(BaseModel):
    """
    A column of data within a Datatable.
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

    values: List[Primitive]
    """The data values of the column."""

    validator: Optional[ArrayValidator]
    """The validator to use to validate data in the column."""
