# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .validator import Validator


class TupleValidator(BaseModel):
    """
    A validator specifying constraints on an array of heterogeneous items.
    """

    id: Optional[str]
    """The identifier for this item"""

    items: Optional[List[Validator]]
    """An array of validators specifying the constraints on each successive item in the array."""
