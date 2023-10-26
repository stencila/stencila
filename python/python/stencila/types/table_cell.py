# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .entity import Entity
from .table_cell_type import TableCellType


@dataclass(init=False)
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

    content: List[Block]
    """Contents of the table cell."""

    def __init__(self, content: List[Block], id: Optional[str] = None, cell_type: Optional[TableCellType] = None, name: Optional[str] = None, column_span: Optional[int] = None, row_span: Optional[int] = None):
        super().__init__(id = id)
        self.cell_type = cell_type
        self.name = name
        self.column_span = column_span
        self.row_span = row_span
        self.content = content
