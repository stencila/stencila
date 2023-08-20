# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .table_cell import TableCell
from .table_row_type import TableRowType


@dataclass(kw_only=True, frozen=True)
class TableRow(Entity):
    """
    A row within a Table.
    """

    type: Literal["TableRow"] = field(default="TableRow", init=False)

    cells: List[TableCell]
    """An array of cells in the row."""

    row_type: Optional[TableRowType] = None
    """The type of row."""
