# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork
from .int_or_str import IntOrStr


@dataclass(kw_only=True, frozen=True)
class PublicationIssue(CreativeWork):
    """
    A part of a successively published publication such as a periodical or publication    volume, often numbered.
    """

    type: Literal["PublicationIssue"] = field(default="PublicationIssue", init=False)

    issue_number: Optional[IntOrStr] = None
    """Identifies the issue of publication; for example, "iii" or "2"."""

    page_start: Optional[IntOrStr] = None
    """The page on which the issue starts; for example "135" or "xiii"."""

    page_end: Optional[IntOrStr] = None
    """The page on which the issue ends; for example "138" or "xvi"."""

    pagination: Optional[str] = None
    """Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55"."""
