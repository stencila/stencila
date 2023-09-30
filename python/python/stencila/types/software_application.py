# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork
SoftwareApplication = ForwardRef("SoftwareApplication")


@dataclass(kw_only=True, frozen=True)
class SoftwareApplication(CreativeWork):
    """
    A software application.
    """

    type: Literal["SoftwareApplication"] = field(default="SoftwareApplication", init=False)

    software_requirements: Optional[List[SoftwareApplication]] = None
    """Requirements for application, including shared libraries that are not included in the application distribution."""

    software_version: Optional[str] = None
    """Version of the software."""
