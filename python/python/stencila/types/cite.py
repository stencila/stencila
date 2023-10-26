# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .citation_intent import CitationIntent
from .citation_mode import CitationMode
from .entity import Entity
from .inline import Inline
from .int_or_str import IntOrStr


@dataclass(init=False)
class Cite(Entity):
    """
    A reference to a `CreativeWork` that is cited in another `CreativeWork`.
    """

    type: Literal["Cite"] = field(default="Cite", init=False)

    target: str
    """The target of the citation (URL or reference ID)."""

    citation_mode: CitationMode
    """Determines how the citation is shown within the surrounding text."""

    citation_intent: Optional[List[CitationIntent]] = None
    """The type/s of the citation, both factually and rhetorically."""

    content: Optional[List[Inline]] = None
    """Optional structured content/text of this citation."""

    page_start: Optional[IntOrStr] = None
    """The page on which the work starts; for example "135" or "xiii"."""

    page_end: Optional[IntOrStr] = None
    """The page on which the work ends; for example "138" or "xvi"."""

    pagination: Optional[str] = None
    """Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55"."""

    citation_prefix: Optional[str] = None
    """Text to show before the citation."""

    citation_suffix: Optional[str] = None
    """Text to show after the citation."""

    def __init__(self, target: str, citation_mode: CitationMode, id: Optional[str] = None, citation_intent: Optional[List[CitationIntent]] = None, content: Optional[List[Inline]] = None, page_start: Optional[IntOrStr] = None, page_end: Optional[IntOrStr] = None, pagination: Optional[str] = None, citation_prefix: Optional[str] = None, citation_suffix: Optional[str] = None):
        super().__init__(id = id)
        self.target = target
        self.citation_mode = citation_mode
        self.citation_intent = citation_intent
        self.content = content
        self.page_start = page_start
        self.page_end = page_end
        self.pagination = pagination
        self.citation_prefix = citation_prefix
        self.citation_suffix = citation_suffix
