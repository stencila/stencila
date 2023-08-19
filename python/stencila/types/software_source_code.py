# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .comment import Comment
from .creative_work_type import CreativeWorkType
from .creative_work_type_or_str import CreativeWorkTypeOrStr
from .date import Date
from .grant_or_monetary_grant import GrantOrMonetaryGrant
from .image_object_or_str import ImageObjectOrStr
from .inline import Inline
from .person import Person
from .person_or_organization import PersonOrOrganization
from .property_value_or_str import PropertyValueOrStr
from .software_application import SoftwareApplication
from .software_source_code_or_software_application_or_str import SoftwareSourceCodeOrSoftwareApplicationOrStr
from .str_or_float import StrOrFloat
from .thing_type import ThingType


class SoftwareSourceCode(BaseModel):
    """
    Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
    """

    id: Optional[str]
    """The identifier for this item"""

    alternate_names: Optional[List[str]]
    """Alternate names (aliases) for the item."""

    description: Optional[List[Block]]
    """A description of the item."""

    identifiers: Optional[List[PropertyValueOrStr]]
    """Any kind of identifier for any kind of Thing."""

    images: Optional[List[ImageObjectOrStr]]
    """Images of the item."""

    name: Optional[str]
    """The name of the item."""

    url: Optional[str]
    """The URL of the item."""

    about: Optional[List[ThingType]]
    """The subject matter of the content."""

    authors: Optional[List[PersonOrOrganization]]
    """The authors of this creative work."""

    comments: Optional[List[Comment]]
    """Comments about this creative work."""

    content: Optional[List[Block]]
    """The structured content of this creative work c.f. property `text`."""

    date_created: Optional[Date]
    """Date/time of creation."""

    date_received: Optional[Date]
    """Date/time that work was received."""

    date_accepted: Optional[Date]
    """Date/time of acceptance."""

    date_modified: Optional[Date]
    """Date/time of most recent modification."""

    date_published: Optional[Date]
    """Date of first publication."""

    editors: Optional[List[Person]]
    """People who edited the `CreativeWork`."""

    funders: Optional[List[PersonOrOrganization]]
    """People or organizations that funded the `CreativeWork`."""

    funded_by: Optional[List[GrantOrMonetaryGrant]]
    """Grants that funded the `CreativeWork`; reverse of `fundedItems`."""

    genre: Optional[List[str]]
    """Genre of the creative work, broadcast channel or group."""

    keywords: Optional[List[str]]
    """Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas."""

    is_part_of: Optional[CreativeWorkType]
    """An item or other CreativeWork that this CreativeWork is a part of."""

    licenses: Optional[List[CreativeWorkTypeOrStr]]
    """License documents that applies to this content, typically indicated by URL."""

    maintainers: Optional[List[PersonOrOrganization]]
    """The people or organizations who maintain this CreativeWork."""

    parts: Optional[List[CreativeWorkType]]
    """Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more."""

    publisher: Optional[PersonOrOrganization]
    """A publisher of the CreativeWork."""

    references: Optional[List[CreativeWorkTypeOrStr]]
    """References to other creative works, such as another publication, web page, scholarly article, etc."""

    text: Optional[str]
    """The textual content of this creative work."""

    title: Optional[List[Inline]]
    """The title of the creative work."""

    version: Optional[StrOrFloat]
    """The version of the creative work."""

    code_repository: Optional[str]
    """Link to the repository where the un-compiled, human readable code and related code is located."""

    code_sample_type: Optional[str]
    """What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template."""

    programming_language: Optional[str]
    """The computer programming language."""

    runtime_platform: Optional[List[str]]
    """Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0)."""

    software_requirements: Optional[List[SoftwareSourceCodeOrSoftwareApplicationOrStr]]
    """Dependency requirements for the software."""

    target_products: Optional[List[SoftwareApplication]]
    """Target operating system or product to which the code applies."""
