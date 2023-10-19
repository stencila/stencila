# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

CreativeWorkType = ForwardRef("CreativeWorkType")
Text = ForwardRef("Text")


CreativeWorkTypeOrText = Union[
    CreativeWorkType,
    Text,
]
"""
`CreativeWorkType` or `Text`
"""
