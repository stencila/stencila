# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cord import Cord
from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class CodeStatic(Entity):
    """
    Abstract base type for non-executable code nodes (e.g. `CodeBlock`).
    """

    type: Literal["CodeStatic"] = field(default="CodeStatic", init=False)

    code: Cord
    """The code."""

    programming_language: Optional[str] = None
    """The programming language of the code."""
