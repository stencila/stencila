# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .parameter import Parameter
from .validator import Validator


class Function(BaseModel):
    """
    A function with a name, which might take Parameters and return a value of a certain type.
    """

    id: Optional[str]
    """The identifier for this item"""

    name: str
    """The name of the function."""

    parameters: List[Parameter]
    """The parameters of the function."""

    returns: Optional[Validator]
    """The return type of the function."""
