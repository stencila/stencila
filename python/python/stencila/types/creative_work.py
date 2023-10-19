# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
Comment = ForwardRef("Comment")
from .creative_work_type import CreativeWorkType
from .creative_work_type_or_text import CreativeWorkTypeOrText
from .date import Date
from .grant_or_monetary_grant import GrantOrMonetaryGrant
from .inline import Inline
from .person import Person
from .person_or_organization import PersonOrOrganization
from .person_or_organization_or_software_application import PersonOrOrganizationOrSoftwareApplication
from .str_or_float import StrOrFloat
from .text import Text
from .thing import Thing
from .thing_type import ThingType


@dataclass(kw_only=True, frozen=True)
class CreativeWork(Thing):
    """
    A creative work, including books, movies, photographs, software programs, etc.
    """

    type: Literal["CreativeWork"] = field(default="CreativeWork", init=False)

    about: Optional[List[ThingType]] = None
    """The subject matter of the content."""

    abstract: Optional[List[Block]] = None
    """A a short description that summarizes a `CreativeWork`."""

    authors: Optional[List[PersonOrOrganization]] = None
    """The authors of the `CreativeWork`."""

    contributors: Optional[List[PersonOrOrganizationOrSoftwareApplication]] = None
    """A secondary contributor to the `CreativeWork`."""

    editors: Optional[List[Person]] = None
    """People who edited the `CreativeWork`."""

    maintainers: Optional[List[PersonOrOrganization]] = None
    """The maintainers of the `CreativeWork`."""

    comments: Optional[List[Comment]] = None
    """Comments about this creative work."""

    date_created: Optional[Date] = None
    """Date/time of creation."""

    date_received: Optional[Date] = None
    """Date/time that work was received."""

    date_accepted: Optional[Date] = None
    """Date/time of acceptance."""

    date_modified: Optional[Date] = None
    """Date/time of most recent modification."""

    date_published: Optional[Date] = None
    """Date of first publication."""

    funders: Optional[List[PersonOrOrganization]] = None
    """People or organizations that funded the `CreativeWork`."""

    funded_by: Optional[List[GrantOrMonetaryGrant]] = None
    """Grants that funded the `CreativeWork`; reverse of `fundedItems`."""

    genre: Optional[List[str]] = None
    """Genre of the creative work, broadcast channel or group."""

    keywords: Optional[List[str]] = None
    """Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas."""

    is_part_of: Optional[CreativeWorkType] = None
    """An item or other CreativeWork that this CreativeWork is a part of."""

    licenses: Optional[List[CreativeWorkTypeOrText]] = None
    """License documents that applies to this content, typically indicated by URL."""

    parts: Optional[List[CreativeWorkType]] = None
    """Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more."""

    publisher: Optional[PersonOrOrganization] = None
    """A publisher of the CreativeWork."""

    references: Optional[List[CreativeWorkTypeOrText]] = None
    """References to other creative works, such as another publication, web page, scholarly article, etc."""

    text: Optional[Text] = None
    """The textual content of this creative work."""

    title: Optional[List[Inline]] = None
    """The title of the creative work."""

    version: Optional[StrOrFloat] = None
    """The version of the creative work."""
