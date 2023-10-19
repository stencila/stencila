# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .creative_work import CreativeWork
from .table_row import TableRow


@dataclass(kw_only=True, frozen=True)
class Table(CreativeWork):
    """
    A table.
    """

    type: Literal["Table"] = field(default="Table", init=False)

    caption: Optional[List[Block]] = None
    """A caption for the table."""

    label: Optional[str] = None
    """A short label for the table."""

    rows: List[TableRow]
    """Rows of cells in the table."""
