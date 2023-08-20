# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork
from .date import Date


@dataclass(kw_only=True, frozen=True)
class Periodical(CreativeWork):
    """
    A periodical publication.
    """

    type: Literal["Periodical"] = field(default="Periodical", init=False)

    date_start: Optional[Date] = None
    """The date this Periodical was first published."""

    date_end: Optional[Date] = None
    """The date this Periodical ceased publication."""

    issns: Optional[List[str]] = None
    """The International Standard Serial Number(s) (ISSN) that identifies this serial publication."""
