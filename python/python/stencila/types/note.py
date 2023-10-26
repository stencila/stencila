# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .entity import Entity
from .note_type import NoteType


@dataclass(init=False)
class Note(Entity):
    """
    Additional content which is not part of the main content of a document.
    """

    type: Literal["Note"] = field(default="Note", init=False)

    note_type: NoteType
    """Determines where the note content is displayed within the document."""

    content: List[Block]
    """Content of the note, usually a paragraph."""

    def __init__(self, note_type: NoteType, content: List[Block], id: Optional[str] = None):
        super().__init__(id = id)
        self.note_type = note_type
        self.content = content
