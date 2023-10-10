# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .claim_type import ClaimType
from .creative_work import CreativeWork


@dataclass(kw_only=True, frozen=True)
class Claim(CreativeWork):
    """
    A claim represents specific reviewable facts or statements.
    """

    type: Literal["Claim"] = field(default="Claim", init=False)

    claim_type: ClaimType
    """The type of the claim."""

    label: Optional[str] = None
    """A short label for the claim."""

    content: List[Block]
    """Content of the claim, usually a single paragraph."""
