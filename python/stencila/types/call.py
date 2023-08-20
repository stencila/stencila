# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .call_argument import CallArgument
from .include import Include


@dataclass(kw_only=True, frozen=True)
class Call(Include):
    """
    Call another document, optionally with arguments, and include its executed content.
    """

    type: Literal["Call"] = field(default="Call", init=False)

    arguments: List[CallArgument]
    """The value of the source document's parameters to call it with"""
