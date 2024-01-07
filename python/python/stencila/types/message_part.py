# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

AudioObject = ForwardRef("AudioObject")
ImageObject = ForwardRef("ImageObject")
VideoObject = ForwardRef("VideoObject")


MessagePart = Union[
    str,
    ImageObject,
    AudioObject,
    VideoObject,
]
"""
A union type for a part of a message.
"""
