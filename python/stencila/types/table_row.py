# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .table_cell import TableCell
from .table_row_type import TableRowType


class TableRow(BaseModel):
    """
    A row within a Table.
    """

    id: Optional[str]
    """The identifier for this item"""

    cells: List[TableCell]
    """An array of cells in the row."""

    row_type: Optional[TableRowType]
    """The type of row."""
