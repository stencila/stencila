# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Underline(Mark):
    """
    Inline text that is underlined.
    """

    type: Literal["Underline"] = field(default="Underline", init=False)
