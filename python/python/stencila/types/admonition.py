# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .admonition_type import AdmonitionType
from .block import Block
from .entity import Entity
from .inline import Inline


@dataclass(init=False)
class Admonition(Entity):
    """
    A admonition within a document.
    """

    type: Literal["Admonition"] = field(default="Admonition", init=False)

    admonition_type: AdmonitionType
    """The type of admonition."""

    title: Optional[List[Inline]] = None
    """The title of the admonition."""

    is_folded: Optional[bool] = None
    """Whether the admonition is folded."""

    content: List[Block]
    """The content within the section."""

    def __init__(self, admonition_type: AdmonitionType, content: List[Block], id: Optional[str] = None, title: Optional[List[Inline]] = None, is_folded: Optional[bool] = None):
        super().__init__(id = id)
        self.admonition_type = admonition_type
        self.title = title
        self.is_folded = is_folded
        self.content = content
