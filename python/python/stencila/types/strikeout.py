# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline
from .mark import Mark


@dataclass(init=False)
class Strikeout(Mark):
    """
    Content that is marked as struck out.
    """

    type: Literal["Strikeout"] = field(default="Strikeout", init=False)

    def __init__(self, content: List[Inline], id: Optional[str] = None):
        super().__init__(id = id, content = content)
        
