# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cite import Cite
from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class CiteGroup(Entity):
    """
    A group of `Cite` nodes.
    """

    type: Literal["CiteGroup"] = field(default="CiteGroup", init=False)

    items: List[Cite]
    """One or more `Cite`s to be referenced in the same surrounding text."""
