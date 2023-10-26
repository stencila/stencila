# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
Comment = ForwardRef("Comment")
from .creative_work_type import CreativeWorkType
from .creative_work_type_or_text import CreativeWorkTypeOrText
from .date import Date
from .grant_or_monetary_grant import GrantOrMonetaryGrant
ImageObject = ForwardRef("ImageObject")
from .inline import Inline
from .person import Person
from .person_or_organization import PersonOrOrganization
from .person_or_organization_or_software_application import PersonOrOrganizationOrSoftwareApplication
from .property_value_or_str import PropertyValueOrStr
from .str_or_float import StrOrFloat
from .text import Text
from .thing import Thing
from .thing_type import ThingType


@dataclass(init=False)
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

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, about: Optional[List[ThingType]] = None, abstract: Optional[List[Block]] = None, authors: Optional[List[PersonOrOrganization]] = None, contributors: Optional[List[PersonOrOrganizationOrSoftwareApplication]] = None, editors: Optional[List[Person]] = None, maintainers: Optional[List[PersonOrOrganization]] = None, comments: Optional[List[Comment]] = None, date_created: Optional[Date] = None, date_received: Optional[Date] = None, date_accepted: Optional[Date] = None, date_modified: Optional[Date] = None, date_published: Optional[Date] = None, funders: Optional[List[PersonOrOrganization]] = None, funded_by: Optional[List[GrantOrMonetaryGrant]] = None, genre: Optional[List[str]] = None, keywords: Optional[List[str]] = None, is_part_of: Optional[CreativeWorkType] = None, licenses: Optional[List[CreativeWorkTypeOrText]] = None, parts: Optional[List[CreativeWorkType]] = None, publisher: Optional[PersonOrOrganization] = None, references: Optional[List[CreativeWorkTypeOrText]] = None, text: Optional[Text] = None, title: Optional[List[Inline]] = None, version: Optional[StrOrFloat] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.about = about
        self.abstract = abstract
        self.authors = authors
        self.contributors = contributors
        self.editors = editors
        self.maintainers = maintainers
        self.comments = comments
        self.date_created = date_created
        self.date_received = date_received
        self.date_accepted = date_accepted
        self.date_modified = date_modified
        self.date_published = date_published
        self.funders = funders
        self.funded_by = funded_by
        self.genre = genre
        self.keywords = keywords
        self.is_part_of = is_part_of
        self.licenses = licenses
        self.parts = parts
        self.publisher = publisher
        self.references = references
        self.text = text
        self.title = title
        self.version = version
