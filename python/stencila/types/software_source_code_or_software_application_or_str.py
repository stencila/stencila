# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .software_application import SoftwareApplication
from .software_source_code import SoftwareSourceCode


SoftwareSourceCodeOrSoftwareApplicationOrStr = Union[
    SoftwareSourceCode,
    SoftwareApplication,
    str,
]
"""
`SoftwareSourceCode` or `SoftwareApplication` or `str`
"""
