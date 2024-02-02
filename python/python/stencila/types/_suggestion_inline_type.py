# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

DeleteInline = ForwardRef("DeleteInline")
InsertInline = ForwardRef("InsertInline")
ModifyInline = ForwardRef("ModifyInline")
ReplaceInline = ForwardRef("ReplaceInline")


SuggestionInlineType = Union[
    DeleteInline,
    InsertInline,
    ModifyInline,
    ReplaceInline,
]
"""
Union type for all types that are descended from `SuggestionInline`
"""
