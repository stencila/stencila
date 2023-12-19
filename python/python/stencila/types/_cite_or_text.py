# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Cite = ForwardRef("Cite")
Text = ForwardRef("Text")


CiteOrText = Union[
    Cite,
    Text,
]
"""
`Cite` or `Text`
"""
