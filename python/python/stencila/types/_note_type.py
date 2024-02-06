# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class NoteType(StrEnum):
    """
    The type of a `Note` which determines where the note content is displayed within the document.
    """

    Footnote = "Footnote"
    Endnote = "Endnote"
    Sidenote = "Sidenote"
