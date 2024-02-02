# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

DeleteBlock = ForwardRef("DeleteBlock")
InsertBlock = ForwardRef("InsertBlock")
ModifyBlock = ForwardRef("ModifyBlock")
ReplaceBlock = ForwardRef("ReplaceBlock")


SuggestionBlockType = Union[
    DeleteBlock,
    InsertBlock,
    ModifyBlock,
    ReplaceBlock,
]
"""
Union type for all types that are descended from `SuggestionBlock`
"""
