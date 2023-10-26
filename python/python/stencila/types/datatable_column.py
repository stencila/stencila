# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .array_validator import ArrayValidator
ImageObject = ForwardRef("ImageObject")
from .primitive import Primitive
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
class DatatableColumn(Thing):
    """
    A column of data within a `Datatable`.
    """

    type: Literal["DatatableColumn"] = field(default="DatatableColumn", init=False)

    values: List[Primitive]
    """The data values of the column."""

    validator: Optional[ArrayValidator] = None
    """The validator to use to validate data in the column."""

    def __init__(self, name: str, values: List[Primitive], id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, url: Optional[str] = None, validator: Optional[ArrayValidator] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.values = values
        self.validator = validator
