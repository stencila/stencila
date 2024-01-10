# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class LabelType(StrEnum):
    """
    Indicates how a block (usually a `CodeChunk`) should be automatically labelled.
    """

    FigureLabel = "FigureLabel"
    TableLabel = "TableLabel"
