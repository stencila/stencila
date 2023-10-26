# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .claim_type import ClaimType
Comment = ForwardRef("Comment")
from .creative_work import CreativeWork
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
from .thing_type import ThingType


@dataclass(init=False)
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

    def __init__(self, claim_type: ClaimType, content: List[Block], id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, about: Optional[List[ThingType]] = None, abstract: Optional[List[Block]] = None, authors: Optional[List[PersonOrOrganization]] = None, contributors: Optional[List[PersonOrOrganizationOrSoftwareApplication]] = None, editors: Optional[List[Person]] = None, maintainers: Optional[List[PersonOrOrganization]] = None, comments: Optional[List[Comment]] = None, date_created: Optional[Date] = None, date_received: Optional[Date] = None, date_accepted: Optional[Date] = None, date_modified: Optional[Date] = None, date_published: Optional[Date] = None, funders: Optional[List[PersonOrOrganization]] = None, funded_by: Optional[List[GrantOrMonetaryGrant]] = None, genre: Optional[List[str]] = None, keywords: Optional[List[str]] = None, is_part_of: Optional[CreativeWorkType] = None, licenses: Optional[List[CreativeWorkTypeOrText]] = None, parts: Optional[List[CreativeWorkType]] = None, publisher: Optional[PersonOrOrganization] = None, references: Optional[List[CreativeWorkTypeOrText]] = None, text: Optional[Text] = None, title: Optional[List[Inline]] = None, version: Optional[StrOrFloat] = None, label: Optional[str] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url, about = about, abstract = abstract, authors = authors, contributors = contributors, editors = editors, maintainers = maintainers, comments = comments, date_created = date_created, date_received = date_received, date_accepted = date_accepted, date_modified = date_modified, date_published = date_published, funders = funders, funded_by = funded_by, genre = genre, keywords = keywords, is_part_of = is_part_of, licenses = licenses, parts = parts, publisher = publisher, references = references, text = text, title = title, version = version)
        self.claim_type = claim_type
        self.label = label
        self.content = content
