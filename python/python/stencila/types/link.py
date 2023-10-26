# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .inline import Inline


@dataclass(init=False)
class Link(Entity):
    """
    A hyperlink to other pages, sections within the same document, resources, or any URL.
    """

    type: Literal["Link"] = field(default="Link", init=False)

    content: List[Inline]
    """The textual content of the link."""

    target: str
    """The target of the link."""

    title: Optional[str] = None
    """A title for the link."""

    rel: Optional[str] = None
    """The relation between the target and the current thing."""

    def __init__(self, content: List[Inline], target: str, id: Optional[str] = None, title: Optional[str] = None, rel: Optional[str] = None):
        super().__init__(id = id)
        self.content = content
        self.target = target
        self.title = title
        self.rel = rel
