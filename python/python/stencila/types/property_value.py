# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
from .primitive import Primitive
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
class PropertyValue(Thing):
    """
    A property-value pair.
    """

    type: Literal["PropertyValue"] = field(default="PropertyValue", init=False)

    property_id: Optional[str] = None
    """A commonly used identifier for the characteristic represented by the property."""

    value: Primitive
    """The value of the property."""

    def __init__(self, value: Primitive, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, property_id: Optional[str] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.property_id = property_id
        self.value = value
