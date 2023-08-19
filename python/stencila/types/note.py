# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .note_type import NoteType


class Note(BaseModel):
    """
    Additional content which is not part of the main content of a document.
    """

    id: Optional[str]
    """The identifier for this item"""

    note_type: NoteType
    """Determines where the note content is displayed within the document."""

    content: List[Block]
    """Content of the note, usually a paragraph."""
