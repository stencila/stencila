# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class Enumeration(Thing):
    """
    Lists or enumerations, for example, a list of cuisines or music genres, etc.
    """

    type: Literal["Enumeration"] = field(default="Enumeration", init=False)
