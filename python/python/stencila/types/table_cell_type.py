# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class TableCellType(StrEnum):
    """
    Indicates whether the cell is a header or data.
    """

    DataCell = "DataCell"
    HeaderCell = "HeaderCell"
