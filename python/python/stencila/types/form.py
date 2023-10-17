# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .executable import Executable
from .form_derive_action import FormDeriveAction
from .int_or_str import IntOrStr


@dataclass(kw_only=True, frozen=True)
class Form(Executable):
    """
    A form to batch updates in document parameters.
    """

    type: Literal["Form"] = field(default="Form", init=False)

    content: List[Block]
    """The content within the form, usually containing at least one `Parameter`."""

    derive_from: Optional[str] = None
    """The dotted path to the object (e.g a database table) that the form should be derived from"""

    derive_action: Optional[FormDeriveAction] = None
    """The action (create, update or delete) to derive for the form"""

    derive_item: Optional[IntOrStr] = None
    """An identifier for the item to be the target of Update or Delete actions"""
