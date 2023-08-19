# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .blocks_or_inlines import BlocksOrInlines
from .table_cell_type import TableCellType


class TableCell(BaseModel):
    """
    A cell within a `Table`.
    """

    id: Optional[str]
    """The identifier for this item"""

    name: Optional[str]
    """The name of the cell."""

    colspan: Optional[int]
    """How many columns the cell extends."""

    cell_type: Optional[TableCellType]
    """The type of cell."""

    rowspan: Optional[int]
    """How many columns the cell extends."""

    content: Optional[BlocksOrInlines]
    """Contents of the table cell."""
