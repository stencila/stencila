# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class SectionType(StrEnum):
    """
    The type of a `Section`.
    """

    Main = "Main"
    Header = "Header"
    Footer = "Footer"
    Summary = "Summary"
    Introduction = "Introduction"
    Methods = "Methods"
    Results = "Results"
    Discussion = "Discussion"
    Conclusion = "Conclusion"
