# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._datatable_column_hint import DatatableColumnHint
from ._entity import Entity


@dataclass(init=False)
class DatatableHint(Entity):
    """
    A hint to the structure of a table of data.
    """

    type: Literal["DatatableHint"] = field(default="DatatableHint", init=False)

    rows: int
    """The number of rows of data."""

    columns: List[DatatableColumnHint]
    """A hint for each column of data."""

    def __init__(self, rows: int, columns: List[DatatableColumnHint], id: Optional[str] = None):
        super().__init__(id = id)
        self.rows = rows
        self.columns = columns
