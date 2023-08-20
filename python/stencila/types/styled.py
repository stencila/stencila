# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .code_executable import CodeExecutable


@dataclass(kw_only=True, frozen=True)
class Styled(CodeExecutable):
    """
    An abstract base class for a document node that has styling applied to it and/or its content
    """

    type: Literal["Styled"] = field(default="Styled", init=False)

    css: Optional[str] = None
    """A Cascading Style Sheet (CSS) transpiled from the output of evaluating the `text` property."""

    classes: Optional[List[str]] = None
    """A list of class names associated with the document node"""
