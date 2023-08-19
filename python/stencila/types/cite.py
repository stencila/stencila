# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .citation_intent import CitationIntent
from .citation_mode import CitationMode
from .inline import Inline
from .int_or_str import IntOrStr


class Cite(BaseModel):
    """
    A reference to a CreativeWork that is cited in another CreativeWork.
    """

    id: Optional[str]
    """The identifier for this item"""

    target: str
    """The target of the citation (URL or reference ID)."""

    citation_mode: CitationMode
    """Determines how the citation is shown within the surrounding text."""

    citation_intent: Optional[List[CitationIntent]]
    """The type/s of the citation, both factually and rhetorically."""

    content: Optional[List[Inline]]
    """Optional structured content/text of this citation."""

    page_start: Optional[IntOrStr]
    """The page on which the work starts; for example "135" or "xiii"."""

    page_end: Optional[IntOrStr]
    """The page on which the work ends; for example "138" or "xvi"."""

    pagination: Optional[str]
    """Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55"."""

    citation_prefix: Optional[str]
    """Text to show before the citation."""

    citation_suffix: Optional[str]
    """Text to show after the citation."""
