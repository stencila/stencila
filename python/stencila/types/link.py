# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline


class Link(BaseModel):
    """
    A hyperlink to other pages, sections within the same document, resources, or any URL.
    """

    id: Optional[str]
    """The identifier for this item"""

    content: List[Inline]
    """The textual content of the link."""

    target: str
    """The target of the link."""

    title: Optional[str]
    """A title for the link."""

    rel: Optional[str]
    """The relation between the target and the current thing."""
