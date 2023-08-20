# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class DefinedTerm(Thing):
    """
    A word, name, acronym, phrase, etc. with a formal definition.
    """

    type: Literal["DefinedTerm"] = field(default="DefinedTerm", init=False)

    term_code: Optional[str] = None
    """A code that identifies this DefinedTerm within a DefinedTermSet"""
