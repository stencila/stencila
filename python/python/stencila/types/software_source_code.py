# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork
SoftwareApplication = ForwardRef("SoftwareApplication")
from .software_source_code_or_software_application_or_str import SoftwareSourceCodeOrSoftwareApplicationOrStr


@dataclass(kw_only=True, frozen=True)
class SoftwareSourceCode(CreativeWork):
    """
    Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
    """

    type: Literal["SoftwareSourceCode"] = field(default="SoftwareSourceCode", init=False)

    code_repository: Optional[str] = None
    """Link to the repository where the un-compiled, human readable code and related code is located."""

    code_sample_type: Optional[str] = None
    """What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template."""

    programming_language: Optional[str] = None
    """The computer programming language."""

    runtime_platform: Optional[List[str]] = None
    """Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0)."""

    software_requirements: Optional[List[SoftwareSourceCodeOrSoftwareApplicationOrStr]] = None
    """Dependency requirements for the software."""

    target_products: Optional[List[SoftwareApplication]] = None
    """Target operating system or product to which the code applies."""
