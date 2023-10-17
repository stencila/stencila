# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .array_validator import ArrayValidator
from .primitive import Primitive
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class DatatableColumn(Thing):
    """
    A column of data within a `Datatable`.
    """

    type: Literal["DatatableColumn"] = field(default="DatatableColumn", init=False)

    values: List[Primitive]
    """The data values of the column."""

    validator: Optional[ArrayValidator] = None
    """The validator to use to validate data in the column."""
