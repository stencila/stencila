# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cite import Cite


class CiteGroup(BaseModel):
    """
    A group of Cite nodes.
    """

    id: Optional[str]
    """The identifier for this item"""

    items: List[Cite]
    """One or more `Cite`s to be referenced in the same surrounding text."""
