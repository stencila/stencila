# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

SoftwareApplication = ForwardRef("SoftwareApplication")
SoftwareSourceCode = ForwardRef("SoftwareSourceCode")


SoftwareSourceCodeOrSoftwareApplicationOrStr = Union[
    SoftwareSourceCode,
    SoftwareApplication,
    str,
]
"""
`SoftwareSourceCode` or `SoftwareApplication` or `str`
"""
