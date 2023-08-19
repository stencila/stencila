# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .execution_digest import ExecutionDigest


class MathBlock(BaseModel):
    """
    A block of math, e.g an equation, to be treated as block content.
    """

    id: Optional[str]
    """The identifier for this item"""

    math_language: str
    """The language used for the equation e.g tex, mathml, asciimath."""

    code: str
    """The code of the equation in the `mathLanguage`."""

    compile_digest: Optional[ExecutionDigest]
    """A digest of the `code` and `mathLanguage` used to avoid unnecessary transpilation to MathML"""

    errors: Optional[List[str]]
    """Errors that occurred when parsing the math equation."""

    mathml: Optional[str]
    """The MathML transpiled from the `code`"""

    label: Optional[str]
    """A short label for the math block."""
