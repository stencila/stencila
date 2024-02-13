# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity
from ._primitive import Primitive


@dataclass(init=False)
class DatatableColumnHint(Entity):
    """
    A hint to the type and values in a `DatatableColumn`.
    """

    type: Literal["DatatableColumnHint"] = field(default="DatatableColumnHint", init=False)

    name: str
    """The name of the column."""

    item_type: str
    """The type of items in the column."""

    minimum: Optional[Primitive] = None
    """The minimum value in the column."""

    maximum: Optional[Primitive] = None
    """The maximum value in the column."""

    nulls: Optional[int] = None
    """The number of `Null` values in the column."""

    def __init__(self, name: str, item_type: str, id: Optional[str] = None, minimum: Optional[Primitive] = None, maximum: Optional[Primitive] = None, nulls: Optional[int] = None):
        super().__init__(id = id)
        self.name = name
        self.item_type = item_type
        self.minimum = minimum
        self.maximum = maximum
        self.nulls = nulls
