# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .code_executable import CodeExecutable
from .node import Node


@dataclass(kw_only=True, frozen=True)
class CodeChunk(CodeExecutable):
    """
    A executable chunk of code.
    """

    type: Literal["CodeChunk"] = field(default="CodeChunk", init=False)

    execution_pure: Optional[bool] = None
    """Whether the code should be treated as side-effect free when executed."""

    outputs: Optional[List[Node]] = None
    """Outputs from executing the chunk."""

    label: Optional[str] = None
    """A short label for the CodeChunk."""

    caption: Optional[List[Block]] = None
    """A caption for the CodeChunk."""
