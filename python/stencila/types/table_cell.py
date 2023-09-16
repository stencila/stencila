# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .blocks_or_inlines import BlocksOrInlines
from .entity import Entity
from .table_cell_type import TableCellType


@dataclass(kw_only=True, frozen=True)
class TableCell(Entity):
    """
    A cell within a `Table`.
    """

    type: Literal["TableCell"] = field(default="TableCell", init=False)

    cell_type: Optional[TableCellType] = None
    """The type of cell."""

    name: Optional[str] = None
    """The name of the cell."""

    column_span: Optional[int] = None
    """How many columns the cell extends."""

    row_span: Optional[int] = None
    """How many columns the cell extends."""

    content: Optional[BlocksOrInlines] = None
    """Contents of the table cell."""
