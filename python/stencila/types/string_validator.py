# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class StringValidator(Entity):
    """
    A schema specifying constraints on a string node.
    """

    type: Literal["StringValidator"] = field(default="StringValidator", init=False)

    min_length: Optional[int] = None
    """The minimum length for a string node."""

    max_length: Optional[int] = None
    """The maximum length for a string node."""

    pattern: Optional[str] = None
    """A regular expression that a string node must match."""
