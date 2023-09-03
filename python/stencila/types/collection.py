# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork


@dataclass(kw_only=True, frozen=True)
class Collection(CreativeWork):
    """
    A collection of CreativeWorks or other artifacts.
    """

    type: Literal["Collection"] = field(default="Collection", init=False)
