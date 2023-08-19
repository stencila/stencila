# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .execution_digest import ExecutionDigest


class MathFragment(BaseModel):
    """
    A fragment of math, e.g a variable name, to be treated as inline content.
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
