# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .table_cell import TableCell
from .table_row_type import TableRowType


@dataclass(init=False)
class TableRow(Entity):
    """
    A row within a Table.
    """

    type: Literal["TableRow"] = field(default="TableRow", init=False)

    cells: List[TableCell]
    """An array of cells in the row."""

    row_type: Optional[TableRowType] = None
    """The type of row."""

    def __init__(self, cells: List[TableCell], id: Optional[str] = None, row_type: Optional[TableRowType] = None):
        super().__init__(id = id)
        self.cells = cells
        self.row_type = row_type
