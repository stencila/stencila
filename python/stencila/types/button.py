# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .code_executable import CodeExecutable


@dataclass(kw_only=True, frozen=True)
class Button(CodeExecutable):
    """
    A button.
    """

    type: Literal["Button"] = field(default="Button", init=False)

    name: str
    """The name of the variable associated with the button."""

    label: Optional[str] = None
    """A label for the button"""

    is_disabled: Optional[bool] = None
    """Whether the button is currently disabled"""
