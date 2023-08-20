# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork
from .datatable_column import DatatableColumn


@dataclass(kw_only=True, frozen=True)
class Datatable(CreativeWork):
    """
    A table of data.
    """

    type: Literal["Datatable"] = field(default="Datatable", init=False)

    columns: List[DatatableColumn]
    """The columns of data."""
