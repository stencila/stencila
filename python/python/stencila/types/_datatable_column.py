# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._array_validator import ArrayValidator
from ._entity import Entity
from ._primitive import Primitive


@dataclass(init=False)
class DatatableColumn(Entity):
    """
    A column of data within a `Datatable`.
    """

    type: Literal["DatatableColumn"] = field(default="DatatableColumn", init=False)

    name: str
    """The name of the column."""

    values: List[Primitive]
    """The data values of the column."""

    validator: Optional[ArrayValidator] = None
    """The validator to use to validate data in the column."""

    def __init__(self, name: str, values: List[Primitive], id: Optional[str] = None, validator: Optional[ArrayValidator] = None):
        super().__init__(id = id)
        self.name = name
        self.values = values
        self.validator = validator
