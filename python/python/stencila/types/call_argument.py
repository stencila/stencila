# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .parameter import Parameter


@dataclass(kw_only=True, frozen=True)
class CallArgument(Parameter):
    """
    The value of a `Parameter` to call a document with.
    """

    type: Literal["CallArgument"] = field(default="CallArgument", init=False)

    code: str
    """The code to be evaluated for the parameter."""

    programming_language: Optional[str] = None
    """The programming language of the code."""
