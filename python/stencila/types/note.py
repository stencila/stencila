# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .entity import Entity
from .note_type import NoteType


@dataclass(kw_only=True, frozen=True)
class Note(Entity):
    """
    Additional content which is not part of the main content of a document.
    """

    type: Literal["Note"] = field(default="Note", init=False)

    note_type: NoteType
    """Determines where the note content is displayed within the document."""

    content: List[Block]
    """Content of the note, usually a paragraph."""
