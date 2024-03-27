# Generated file; do not edit. See the Rust `schema-gen` crate.
# We override the Literal `type` in each class so...
# pyright: reportIncompatibleVariableOverride=false
from __future__ import annotations

import sys
from dataclasses import dataclass, fields, is_dataclass
from typing import Literal, Union

if sys.version_info >= (3, 11):
    from enum import StrEnum
else:
    from strenum import StrEnum

# Primitive types
UnsignedInteger = int
Cord = str
Array = list
Primitive = Union[
    None,
    bool,
    int,
    UnsignedInteger,
    float,
    str,
    Array,
    "Object",
]

Object = dict[str, Primitive]


class _Base:
    """Provide a base class with a simplified repr that ignores None values."""

    def __repr__(self):
        if not is_dataclass(self):
            raise TypeError("_Base should only be used with dataclasses")

        field_names = [f.name for f in fields(self)]
        valid_fields = {
            name: getattr(self, name)
            for name in field_names
            if getattr(self, name) is not None
        }
        repr_str = (
            f"{self.__class__.__name__}("  # type: ignore
            + ", ".join([f"{key}={value!r}" for key, value in valid_fields.items()])
            + ")"
        )
        return repr_str


class AdmonitionType(StrEnum):
    """
    The type of an `Admonition`.
    """

    Note = "Note"
    Info = "Info"
    Tip = "Tip"
    Important = "Important"
    Success = "Success"
    Failure = "Failure"
    Warning = "Warning"
    Danger = "Danger"
    Error = "Error"


class AuthorRoleName(StrEnum):
    """
    A `roleName` for an `AuthorRole`.
    """

    Writer = "Writer"
    Verifier = "Verifier"
    Instructor = "Instructor"
    Prompter = "Prompter"
    Generator = "Generator"


class AutomaticExecution(StrEnum):
    """
    Under which circumstances the document node should be automatically executed.
    """

    Never = "Never"
    Needed = "Needed"
    Always = "Always"


class CitationIntent(StrEnum):
    """
    The type or nature of a citation, both factually and rhetorically.
    """

    AgreesWith = "AgreesWith"
    CitesAsAuthority = "CitesAsAuthority"
    CitesAsDataSource = "CitesAsDataSource"
    CitesAsEvidence = "CitesAsEvidence"
    CitesAsMetadataDocument = "CitesAsMetadataDocument"
    CitesAsPotentialSolution = "CitesAsPotentialSolution"
    CitesAsRecommendedReading = "CitesAsRecommendedReading"
    CitesAsRelated = "CitesAsRelated"
    CitesAsSourceDocument = "CitesAsSourceDocument"
    CitesForInformation = "CitesForInformation"
    Compiles = "Compiles"
    Confirms = "Confirms"
    ContainsAssertionFrom = "ContainsAssertionFrom"
    Corrects = "Corrects"
    Credits = "Credits"
    Critiques = "Critiques"
    Derides = "Derides"
    Describes = "Describes"
    DisagreesWith = "DisagreesWith"
    Discusses = "Discusses"
    Disputes = "Disputes"
    Documents = "Documents"
    Extends = "Extends"
    GivesBackgroundTo = "GivesBackgroundTo"
    GivesSupportTo = "GivesSupportTo"
    HasReplyFrom = "HasReplyFrom"
    IncludesExcerptFrom = "IncludesExcerptFrom"
    IncludesQuotationFrom = "IncludesQuotationFrom"
    IsAgreedWithBy = "IsAgreedWithBy"
    IsCitedAsAuthorityBy = "IsCitedAsAuthorityBy"
    IsCitedAsDataSourceBy = "IsCitedAsDataSourceBy"
    IsCitedAsEvidenceBy = "IsCitedAsEvidenceBy"
    IsCitedAsMetadataDocumentBy = "IsCitedAsMetadataDocumentBy"
    IsCitedAsPotentialSolutionBy = "IsCitedAsPotentialSolutionBy"
    IsCitedAsRecommendedReadingBy = "IsCitedAsRecommendedReadingBy"
    IsCitedAsRelatedBy = "IsCitedAsRelatedBy"
    IsCitedAsSourceDocumentBy = "IsCitedAsSourceDocumentBy"
    IsCitedBy = "IsCitedBy"
    IsCitedForInformationBy = "IsCitedForInformationBy"
    IsCompiledBy = "IsCompiledBy"
    IsConfirmedBy = "IsConfirmedBy"
    IsCorrectedBy = "IsCorrectedBy"
    IsCreditedBy = "IsCreditedBy"
    IsCritiquedBy = "IsCritiquedBy"
    IsDeridedBy = "IsDeridedBy"
    IsDescribedBy = "IsDescribedBy"
    IsDisagreedWithBy = "IsDisagreedWithBy"
    IsDiscussedBy = "IsDiscussedBy"
    IsDisputedBy = "IsDisputedBy"
    IsDocumentedBy = "IsDocumentedBy"
    IsExtendedBy = "IsExtendedBy"
    IsLinkedToBy = "IsLinkedToBy"
    IsParodiedBy = "IsParodiedBy"
    IsPlagiarizedBy = "IsPlagiarizedBy"
    IsQualifiedBy = "IsQualifiedBy"
    IsRefutedBy = "IsRefutedBy"
    IsRetractedBy = "IsRetractedBy"
    IsReviewedBy = "IsReviewedBy"
    IsRidiculedBy = "IsRidiculedBy"
    IsSpeculatedOnBy = "IsSpeculatedOnBy"
    IsSupportedBy = "IsSupportedBy"
    IsUpdatedBy = "IsUpdatedBy"
    Likes = "Likes"
    LinksTo = "LinksTo"
    ObtainsBackgroundFrom = "ObtainsBackgroundFrom"
    ObtainsSupportFrom = "ObtainsSupportFrom"
    Parodies = "Parodies"
    Plagiarizes = "Plagiarizes"
    ProvidesAssertionFor = "ProvidesAssertionFor"
    ProvidesConclusionsFor = "ProvidesConclusionsFor"
    ProvidesDataFor = "ProvidesDataFor"
    ProvidesExcerptFor = "ProvidesExcerptFor"
    ProvidesMethodFor = "ProvidesMethodFor"
    ProvidesQuotationFor = "ProvidesQuotationFor"
    Qualifies = "Qualifies"
    Refutes = "Refutes"
    RepliesTo = "RepliesTo"
    Retracts = "Retracts"
    Reviews = "Reviews"
    Ridicules = "Ridicules"
    SharesAuthorInstitutionWith = "SharesAuthorInstitutionWith"
    SharesAuthorWith = "SharesAuthorWith"
    SharesFundingAgencyWith = "SharesFundingAgencyWith"
    SharesJournalWith = "SharesJournalWith"
    SharesPublicationVenueWith = "SharesPublicationVenueWith"
    SpeculatesOn = "SpeculatesOn"
    Supports = "Supports"
    Updates = "Updates"
    UsesConclusionsFrom = "UsesConclusionsFrom"
    UsesDataFrom = "UsesDataFrom"
    UsesMethodIn = "UsesMethodIn"


class CitationMode(StrEnum):
    """
    The mode of a `Cite`.
    """

    Parenthetical = "Parenthetical"
    Narrative = "Narrative"
    NarrativeAuthor = "NarrativeAuthor"


class ClaimType(StrEnum):
    """
    The type of a `Claim`.
    """

    Statement = "Statement"
    Theorem = "Theorem"
    Lemma = "Lemma"
    Proof = "Proof"
    Postulate = "Postulate"
    Hypothesis = "Hypothesis"
    Proposition = "Proposition"
    Corollary = "Corollary"


class ExecutionDependantRelation(StrEnum):
    """
    The relation between a node and its execution dependant.
    """

    Assigns = "Assigns"
    Alters = "Alters"
    Declares = "Declares"
    Writes = "Writes"


class ExecutionDependencyRelation(StrEnum):
    """
    The relation between a node and its execution dependency.
    """

    Calls = "Calls"
    Derives = "Derives"
    Imports = "Imports"
    Includes = "Includes"
    Reads = "Reads"
    Uses = "Uses"


class ExecutionRequired(StrEnum):
    """
    Whether, and why, the execution of a node is required or not.
    """

    No = "No"
    NeverExecuted = "NeverExecuted"
    SemanticsChanged = "SemanticsChanged"
    DependenciesChanged = "DependenciesChanged"
    DependenciesFailed = "DependenciesFailed"
    ExecutionFailed = "ExecutionFailed"
    ExecutionCancelled = "ExecutionCancelled"
    ExecutionInterrupted = "ExecutionInterrupted"
    KernelRestarted = "KernelRestarted"
    UserRequested = "UserRequested"


class ExecutionStatus(StrEnum):
    """
    Status of the most recent, including any current, execution of a document node.
    """

    Scheduled = "Scheduled"
    Pending = "Pending"
    Skipped = "Skipped"
    Empty = "Empty"
    Running = "Running"
    Succeeded = "Succeeded"
    Warnings = "Warnings"
    Errors = "Errors"
    Exceptions = "Exceptions"
    Cancelled = "Cancelled"
    Interrupted = "Interrupted"


class FormDeriveAction(StrEnum):
    """
    Indicates the action (create, update or delete) to derive for a `Form`.
    """

    Create = "Create"
    Update = "Update"
    Delete = "Delete"
    UpdateOrDelete = "UpdateOrDelete"


class LabelType(StrEnum):
    """
    Indicates how a block (usually a `CodeChunk`) should be automatically labelled.
    """

    FigureLabel = "FigureLabel"
    TableLabel = "TableLabel"


class ListOrder(StrEnum):
    """
    Indicates how a `List` is ordered.
    """

    Ascending = "Ascending"
    Descending = "Descending"
    Unordered = "Unordered"


class MessageLevel(StrEnum):
    """
    The severity level of a message.
    """

    Trace = "Trace"
    Debug = "Debug"
    Info = "Info"
    Warning = "Warning"
    Error = "Error"
    Exception = "Exception"


class NoteType(StrEnum):
    """
    The type of a `Note` which determines where the note content is displayed within the document.
    """

    Footnote = "Footnote"
    Endnote = "Endnote"
    Sidenote = "Sidenote"


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
    Iteration = "Iteration"


class SuggestionStatus(StrEnum):
    """
    The status of an instruction.
    """

    Proposed = "Proposed"
    Accepted = "Accepted"
    Rejected = "Rejected"


class TableCellType(StrEnum):
    """
    Indicates whether the cell is a header or data.
    """

    DataCell = "DataCell"
    HeaderCell = "HeaderCell"


class TableRowType(StrEnum):
    """
    Indicates whether the row is in the header, body or footer of the table.
    """

    HeaderRow = "HeaderRow"
    BodyRow = "BodyRow"
    FooterRow = "FooterRow"


class TimeUnit(StrEnum):
    """
    A unit in which time can be measured.
    """

    Year = "Year"
    Month = "Month"
    Week = "Week"
    Day = "Day"
    Hour = "Hour"
    Minute = "Minute"
    Second = "Second"
    Millisecond = "Millisecond"
    Microsecond = "Microsecond"
    Nanosecond = "Nanosecond"
    Picosecond = "Picosecond"
    Femtosecond = "Femtosecond"
    Attosecond = "Attosecond"



@dataclass(kw_only=True, repr=False)
class Entity(_Base):
    """
    Abstract base type for compound (ie. non-atomic) nodes.
    """

    type: Literal["Entity"] = "Entity"

    id: str | None = None
    """The identifier for this item."""


@dataclass(kw_only=True, repr=False)
class Thing(Entity):
    """
    The most generic type of item.
    """

    type: Literal["Thing"] = "Thing"

    alternate_names: list[str] | None = None
    """Alternate names (aliases) for the item."""

    description: Text | None = None
    """A description of the item."""

    identifiers: list[PropertyValue | str] | None = None
    """Any kind of identifier for any kind of Thing."""

    images: list[ImageObject] | None = None
    """Images of the item."""

    name: str | None = None
    """The name of the item."""

    url: str | None = None
    """The URL of the item."""


@dataclass(kw_only=True, repr=False)
class CreativeWork(Thing):
    """
    A creative work, including books, movies, photographs, software programs, etc.
    """

    type: Literal["CreativeWork"] = "CreativeWork"

    about: list[ThingType] | None = None
    """The subject matter of the content."""

    abstract: list[Block] | None = None
    """A a short description that summarizes a `CreativeWork`."""

    authors: list[Author] | None = None
    """The authors of the `CreativeWork`."""

    contributors: list[Author] | None = None
    """A secondary contributor to the `CreativeWork`."""

    editors: list[Person] | None = None
    """People who edited the `CreativeWork`."""

    maintainers: list[Person | Organization] | None = None
    """The maintainers of the `CreativeWork`."""

    comments: list[Comment] | None = None
    """Comments about this creative work."""

    date_created: Date | None = None
    """Date/time of creation."""

    date_received: Date | None = None
    """Date/time that work was received."""

    date_accepted: Date | None = None
    """Date/time of acceptance."""

    date_modified: Date | None = None
    """Date/time of most recent modification."""

    date_published: Date | None = None
    """Date of first publication."""

    funders: list[Person | Organization] | None = None
    """People or organizations that funded the `CreativeWork`."""

    funded_by: list[Grant | MonetaryGrant] | None = None
    """Grants that funded the `CreativeWork`; reverse of `fundedItems`."""

    genre: list[str] | None = None
    """Genre of the creative work, broadcast channel or group."""

    keywords: list[str] | None = None
    """Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas."""

    is_part_of: CreativeWorkType | None = None
    """An item or other CreativeWork that this CreativeWork is a part of."""

    licenses: list[CreativeWorkType | Text] | None = None
    """License documents that applies to this content, typically indicated by URL."""

    parts: list[CreativeWorkType] | None = None
    """Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more."""

    publisher: Person | Organization | None = None
    """A publisher of the CreativeWork."""

    references: list[CreativeWorkType | Text] | None = None
    """References to other creative works, such as another publication, web page, scholarly article, etc."""

    text: Text | None = None
    """The textual content of this creative work."""

    title: list[Inline] | None = None
    """The title of the creative work."""

    version: str | float | None = None
    """The version of the creative work."""


@dataclass(kw_only=True, repr=False)
class Executable(Entity):
    """
    Abstract base type for executable nodes (e.g. `CodeChunk`, `CodeExpression`, `Call`).
    """

    type: Literal["Executable"] = "Executable"

    auto_exec: AutomaticExecution | None = None
    """Under which circumstances the code should be automatically executed."""

    compilation_digest: CompilationDigest | None = None
    """A digest of the content, semantics and dependencies of the node."""

    compilation_messages: list[CompilationMessage] | None = None
    """Messages generated while compiling the code."""

    execution_digest: CompilationDigest | None = None
    """The `compilationDigest` of the node when it was last executed."""

    execution_dependencies: list[ExecutionDependency] | None = None
    """The upstream dependencies of this node."""

    execution_dependants: list[ExecutionDependant] | None = None
    """The downstream dependants of this node."""

    execution_tags: list[ExecutionTag] | None = None
    """Tags in the code which affect its execution."""

    execution_count: int | None = None
    """A count of the number of times that the node has been executed."""

    execution_required: ExecutionRequired | None = None
    """Whether, and why, the code requires execution or re-execution."""

    execution_status: ExecutionStatus | None = None
    """Status of the most recent, including any current, execution."""

    execution_actor: str | None = None
    """The id of the actor that the node was last executed by."""

    execution_ended: Timestamp | None = None
    """The timestamp when the last execution ended."""

    execution_duration: Duration | None = None
    """Duration of the last execution."""

    execution_messages: list[ExecutionMessage] | None = None
    """Messages emitted while executing the node."""


@dataclass(kw_only=True, repr=False)
class Suggestion(Entity):
    """
    Abstract base type for nodes that indicate a suggested change to content.
    """

    type: Literal["Suggestion"] = "Suggestion"

    suggestion_status: SuggestionStatus | None = None
    """The status of the suggestion including whether it is proposed, accepted, or rejected."""


@dataclass(kw_only=True, repr=False)
class CodeExecutable(Executable):
    """
    Abstract base type for executable code nodes (e.g. `CodeChunk`).
    """

    type: Literal["CodeExecutable"] = "CodeExecutable"

    code: Cord
    """The code."""

    programming_language: str | None = None
    """The programming language of the code."""

    authors: list[Author] | None = None
    """The authors of the executable code."""


@dataclass(kw_only=True, repr=False)
class CodeStatic(Entity):
    """
    Abstract base type for non-executable code nodes (e.g. `CodeBlock`).
    """

    type: Literal["CodeStatic"] = "CodeStatic"

    code: Cord
    """The code."""

    programming_language: str | None = None
    """The programming language of the code."""

    authors: list[Author] | None = None
    """The authors of the code."""


@dataclass(kw_only=True, repr=False)
class ContactPoint(Thing):
    """
    A contact point, usually within an organization.
    """

    type: Literal["ContactPoint"] = "ContactPoint"

    emails: list[str] | None = None
    """Email address for correspondence."""

    telephone_numbers: list[str] | None = None
    """Telephone numbers for the contact point."""

    available_languages: list[str] | None = None
    """Languages (human not programming) in which it is possible to communicate with the organization/department etc."""


@dataclass(kw_only=True, repr=False)
class Grant(Thing):
    """
    A grant, typically financial or otherwise quantifiable, of resources.
    """

    type: Literal["Grant"] = "Grant"

    funded_items: list[Thing] | None = None
    """Indicates an item funded or sponsored through a Grant."""

    sponsors: list[Person | Organization] | None = None
    """A person or organization that supports a thing through a pledge, promise, or financial contribution."""


@dataclass(kw_only=True, repr=False)
class IncludeBlock(Executable):
    """
    Include block content from an external source (e.g. file, URL).
    """

    type: Literal["IncludeBlock"] = "IncludeBlock"

    source: str
    """The external source of the content, a file path or URL."""

    media_type: str | None = None
    """Media type of the source content."""

    select: str | None = None
    """A query to select a subset of content from the source"""

    content: list[Block] | None = None
    """The structured content decoded from the source."""


@dataclass(kw_only=True, repr=False)
class Instruction(Executable):
    """
    Abstract base type for a document editing instruction.
    """

    type: Literal["Instruction"] = "Instruction"

    messages: list[InstructionMessage]
    """Messages involved in the instruction."""

    candidates: list[str] | None = None
    """A list of candidates for the assignee property."""

    assignee: str | None = None
    """An identifier for the agent assigned to perform the instruction"""

    authors: list[Author] | None = None
    """The authors of the instruction."""


@dataclass(kw_only=True, repr=False)
class Mark(Entity):
    """
    Abstract base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).
    """

    type: Literal["Mark"] = "Mark"

    content: list[Inline]
    """The content that is marked."""


@dataclass(kw_only=True, repr=False)
class Math(Entity):
    """
    Abstract base type for a mathematical variable or equation.
    """

    type: Literal["Math"] = "Math"

    code: Cord
    """The code of the equation in the `mathLanguage`."""

    math_language: str | None = None
    """The language used for the equation e.g tex, mathml, asciimath."""

    authors: list[Author] | None = None
    """The authors of the math."""

    compilation_digest: CompilationDigest | None = None
    """A digest of the `code` and `mathLanguage`."""

    compilation_messages: list[CompilationMessage] | None = None
    """Messages generated while parsing and compiling the math expression."""

    mathml: str | None = None
    """The MathML transpiled from the `code`."""


@dataclass(kw_only=True, repr=False)
class MediaObject(CreativeWork):
    """
    A media object, such as an image, video, or audio object embedded in a web page or a downloadable dataset.
    """

    type: Literal["MediaObject"] = "MediaObject"

    bitrate: float | None = None
    """Bitrate in megabits per second (Mbit/s, Mb/s, Mbps)."""

    content_size: float | None = None
    """File size in megabits (Mbit, Mb)."""

    content_url: str
    """URL for the actual bytes of the media object, for example the image file or video file."""

    embed_url: str | None = None
    """URL that can be used to embed the media on a web page via a specific media player."""

    media_type: str | None = None
    """IANA media type (MIME type)."""


@dataclass(kw_only=True, repr=False)
class NumberValidator(Entity):
    """
    A validator specifying the constraints on a numeric node.
    """

    type: Literal["NumberValidator"] = "NumberValidator"

    minimum: float | None = None
    """The inclusive lower limit for a numeric node."""

    exclusive_minimum: float | None = None
    """The exclusive lower limit for a numeric node."""

    maximum: float | None = None
    """The inclusive upper limit for a numeric node."""

    exclusive_maximum: float | None = None
    """The exclusive upper limit for a numeric node."""

    multiple_of: float | None = None
    """A number that a numeric node must be a multiple of."""


@dataclass(kw_only=True, repr=False)
class Parameter(Executable):
    """
    A parameter of a document.
    """

    type: Literal["Parameter"] = "Parameter"

    name: str
    """The name of the parameter."""

    label: str | None = None
    """A short label for the parameter."""

    value: Node | None = None
    """The current value of the parameter."""

    default: Node | None = None
    """The default value of the parameter."""

    validator: Validator | None = None
    """The validator that the value is validated against."""

    derived_from: str | None = None
    """The dotted path to the object (e.g. a database table column) that the parameter should be derived from"""


@dataclass(kw_only=True, repr=False)
class Role(Entity):
    """
    Represents additional information about a relationship or property.
    """

    type: Literal["Role"] = "Role"


@dataclass(kw_only=True, repr=False)
class Styled(Entity):
    """
    An abstract base class for a document node that has styling applied to it and/or its content.
    """

    type: Literal["Styled"] = "Styled"

    code: Cord
    """The code of the equation in the `styleLanguage`."""

    style_language: str | None = None
    """The language used for the style specification e.g. css, tw"""

    authors: list[Author] | None = None
    """The authors of the styling code."""

    compilation_digest: CompilationDigest | None = None
    """A digest of the `code` and `styleLanguage`."""

    compilation_messages: list[CompilationMessage] | None = None
    """Messages generated while parsing and transpiling the style."""

    css: str | None = None
    """A Cascading Style Sheet (CSS) transpiled from the `code` property."""

    class_list: str | None = None
    """A space separated list of class names associated with the node."""


@dataclass(kw_only=True, repr=False)
class SuggestionBlock(Suggestion):
    """
    Abstract base type for nodes that indicate a suggested change to block content.
    """

    type: Literal["SuggestionBlock"] = "SuggestionBlock"

    content: list[Block]
    """The content that is suggested to be inserted, modified, replaced, or deleted."""


@dataclass(kw_only=True, repr=False)
class SuggestionInline(Suggestion):
    """
    Abstract base type for nodes that indicate a suggested change to inline content.
    """

    type: Literal["SuggestionInline"] = "SuggestionInline"

    content: list[Inline]
    """The content that is suggested to be inserted, modified, replaced, or deleted."""


@dataclass(kw_only=True, repr=False)
class Admonition(Entity):
    """
    A admonition within a document.
    """

    type: Literal["Admonition"] = "Admonition"

    admonition_type: AdmonitionType
    """The type of admonition."""

    title: list[Inline] | None = None
    """The title of the admonition."""

    is_folded: bool | None = None
    """Whether the admonition is folded."""

    content: list[Block]
    """The content within the section."""

    authors: list[Author] | None = None
    """The authors of the admonition."""


@dataclass(kw_only=True, repr=False)
class ArrayHint(Entity):
    """
    A hint to the content of an `Array`.
    """

    type: Literal["ArrayHint"] = "ArrayHint"

    length: int
    """The length (number of items) of the array."""

    item_types: list[str] | None = None
    """The distinct types of the array items."""

    minimum: Primitive | None = None
    """The minimum value in the array."""

    maximum: Primitive | None = None
    """The maximum value in the array."""

    nulls: int | None = None
    """The number of `Null` values in the array."""


@dataclass(kw_only=True, repr=False)
class ArrayValidator(Entity):
    """
    A validator specifying constraints on an array node.
    """

    type: Literal["ArrayValidator"] = "ArrayValidator"

    items_nullable: bool | None = None
    """Whether items can have the value `Node::Null`"""

    items_validator: Validator | None = None
    """Another validator node specifying the constraints on all items in the array."""

    contains: Validator | None = None
    """An array node is valid if at least one of its items is valid against the `contains` schema."""

    min_items: int | None = None
    """An array node is valid if its size is greater than, or equal to, this value."""

    max_items: int | None = None
    """An array node is valid if its size is less than, or equal to, this value."""

    unique_items: bool | None = None
    """A flag to indicate that each value in the array should be unique."""


@dataclass(kw_only=True, repr=False)
class Article(CreativeWork, Executable):
    """
    An article, including news and scholarly articles.
    """

    type: Literal["Article"] = "Article"

    content: list[Block]
    """The content of the article."""

    page_start: int | str | None = None
    """The page on which the article starts; for example "135" or "xiii"."""

    page_end: int | str | None = None
    """The page on which the article ends; for example "138" or "xvi"."""

    pagination: str | None = None
    """Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55"."""


@dataclass(kw_only=True, repr=False)
class AudioObject(MediaObject):
    """
    An audio file.
    """

    type: Literal["AudioObject"] = "AudioObject"

    caption: list[Inline] | None = None
    """The caption for this audio recording."""

    transcript: str | None = None
    """The transcript of this audio recording."""


@dataclass(kw_only=True, repr=False)
class AuthorRole(Role):
    """
    An author and their role.
    """

    type: Literal["AuthorRole"] = "AuthorRole"

    author: Person | Organization | SoftwareApplication
    """The author."""

    role_name: AuthorRoleName
    """A role played by the author."""

    last_modified: Timestamp | None = None
    """Timestamp of most recent modification by the author in the role."""


@dataclass(kw_only=True, repr=False)
class BooleanValidator(Entity):
    """
    A schema specifying that a node must be a boolean value.
    """

    type: Literal["BooleanValidator"] = "BooleanValidator"


@dataclass(kw_only=True, repr=False)
class Brand(Thing):
    """
    A brand used by an organization or person for labeling a product, product group, or similar.
    """

    type: Literal["Brand"] = "Brand"

    logo: ImageObject | None = None
    """A logo associated with the brand."""

    reviews: list[str] | None = None
    """Reviews of the brand."""


@dataclass(kw_only=True, repr=False)
class Button(CodeExecutable):
    """
    A button.
    """

    type: Literal["Button"] = "Button"

    name: str
    """The name of the variable associated with the button."""

    label: str | None = None
    """A label for the button"""

    is_disabled: bool | None = None
    """Whether the button is currently disabled"""


@dataclass(kw_only=True, repr=False)
class CallArgument(Parameter):
    """
    The value of a `Parameter` to call a document with.
    """

    type: Literal["CallArgument"] = "CallArgument"

    code: Cord
    """The code to be evaluated for the parameter."""

    programming_language: str | None = None
    """The programming language of the code."""


@dataclass(kw_only=True, repr=False)
class CallBlock(IncludeBlock):
    """
    Call another document, optionally with arguments, and include its executed content.
    """

    type: Literal["CallBlock"] = "CallBlock"

    arguments: list[CallArgument]
    """The value of the source document's parameters to call it with"""


@dataclass(kw_only=True, repr=False)
class Cite(Entity):
    """
    A reference to a `CreativeWork` that is cited in another `CreativeWork`.
    """

    type: Literal["Cite"] = "Cite"

    target: str
    """The target of the citation (URL or reference ID)."""

    citation_mode: CitationMode
    """Determines how the citation is shown within the surrounding text."""

    citation_intent: list[CitationIntent] | None = None
    """The type/s of the citation, both factually and rhetorically."""

    content: list[Inline] | None = None
    """Optional structured content/text of this citation."""

    page_start: int | str | None = None
    """The page on which the work starts; for example "135" or "xiii"."""

    page_end: int | str | None = None
    """The page on which the work ends; for example "138" or "xvi"."""

    pagination: str | None = None
    """Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55"."""

    citation_prefix: str | None = None
    """Text to show before the citation."""

    citation_suffix: str | None = None
    """Text to show after the citation."""


@dataclass(kw_only=True, repr=False)
class CiteGroup(Entity):
    """
    A group of `Cite` nodes.
    """

    type: Literal["CiteGroup"] = "CiteGroup"

    items: list[Cite]
    """One or more `Cite`s to be referenced in the same surrounding text."""


@dataclass(kw_only=True, repr=False)
class Claim(CreativeWork):
    """
    A claim represents specific reviewable facts or statements.
    """

    type: Literal["Claim"] = "Claim"

    claim_type: ClaimType
    """The type of the claim."""

    label: str | None = None
    """A short label for the claim."""

    content: list[Block]
    """Content of the claim, usually a single paragraph."""


@dataclass(kw_only=True, repr=False)
class CodeBlock(CodeStatic):
    """
    A code block.
    """

    type: Literal["CodeBlock"] = "CodeBlock"


@dataclass(kw_only=True, repr=False)
class CodeChunk(CodeExecutable):
    """
    A executable chunk of code.
    """

    type: Literal["CodeChunk"] = "CodeChunk"

    label_type: LabelType | None = None
    """The type of the label for the chunk."""

    label: str | None = None
    """A short label for the chunk."""

    caption: list[Block] | None = None
    """A caption for the chunk."""

    outputs: list[Node] | None = None
    """Outputs from executing the chunk."""

    execution_pure: bool | None = None
    """Whether the code should be treated as side-effect free when executed."""


@dataclass(kw_only=True, repr=False)
class CodeExpression(CodeExecutable):
    """
    An executable programming code expression.
    """

    type: Literal["CodeExpression"] = "CodeExpression"

    output: Node | None = None
    """The value of the expression when it was last evaluated."""


@dataclass(kw_only=True, repr=False)
class CodeInline(CodeStatic):
    """
    Inline code.
    """

    type: Literal["CodeInline"] = "CodeInline"


@dataclass(kw_only=True, repr=False)
class CodeLocation(Entity):
    """
    The location within some source code.
    """

    type: Literal["CodeLocation"] = "CodeLocation"

    source: str | None = None
    """The source of the code, a file path, label or URL."""

    start_line: UnsignedInteger | None = None
    """The 1-based index if the first line on which the error occurred."""

    start_column: UnsignedInteger | None = None
    """The 1-based index if the first column on which the error occurred."""

    end_line: UnsignedInteger | None = None
    """The 1-based index if the last line on which the error occurred."""

    end_column: UnsignedInteger | None = None
    """The 1-based index if the last column on which the error occurred."""


@dataclass(kw_only=True, repr=False)
class Collection(CreativeWork):
    """
    A collection of CreativeWorks or other artifacts.
    """

    type: Literal["Collection"] = "Collection"


@dataclass(kw_only=True, repr=False)
class Comment(CreativeWork):
    """
    A comment on an item, e.g on a `Article` or `SoftwareSourceCode`.
    """

    type: Literal["Comment"] = "Comment"

    content: list[Block]
    """Content of the comment, usually one or more paragraphs."""

    parent_item: Comment | None = None
    """The parent comment of this comment."""

    comment_aspect: str | None = None
    """The part or facet of the item that is being commented on."""


@dataclass(kw_only=True, repr=False)
class CompilationDigest(Entity):
    """
    A digest of the content, semantics and dependencies of an executable node.
    """

    type: Literal["CompilationDigest"] = "CompilationDigest"

    state_digest: UnsignedInteger
    """A digest of the state of a node."""

    semantic_digest: UnsignedInteger | None = None
    """A digest of the semantics of the node with respect to the dependency graph."""

    dependencies_digest: UnsignedInteger | None = None
    """A digest of the semantic digests of the dependencies of a node."""

    dependencies_stale: UnsignedInteger | None = None
    """A count of the number of dependencies that are stale."""

    dependencies_failed: UnsignedInteger | None = None
    """A count of the number of dependencies that failed."""


@dataclass(kw_only=True, repr=False)
class CompilationMessage(Entity):
    """
    An error, warning or log message generated during compilation.
    """

    type: Literal["CompilationMessage"] = "CompilationMessage"

    level: MessageLevel
    """The severity level of the message."""

    message: str
    """The text of the message."""

    error_type: str | None = None
    """The type of error e.g. "SyntaxError", "ZeroDivisionError"."""

    code_location: CodeLocation | None = None
    """The location that the error occurred."""


@dataclass(kw_only=True, repr=False)
class ConstantValidator(Entity):
    """
    A validator specifying a constant value that a node must have.
    """

    type: Literal["ConstantValidator"] = "ConstantValidator"

    value: Node
    """The value that the node must have."""


@dataclass(kw_only=True, repr=False)
class Datatable(CreativeWork):
    """
    A table of data.
    """

    type: Literal["Datatable"] = "Datatable"

    columns: list[DatatableColumn]
    """The columns of data."""


@dataclass(kw_only=True, repr=False)
class DatatableColumn(Entity):
    """
    A column of data within a `Datatable`.
    """

    type: Literal["DatatableColumn"] = "DatatableColumn"

    name: str
    """The name of the column."""

    values: list[Primitive]
    """The data values of the column."""

    validator: ArrayValidator | None = None
    """The validator to use to validate data in the column."""


@dataclass(kw_only=True, repr=False)
class DatatableColumnHint(Entity):
    """
    A hint to the type and values in a `DatatableColumn`.
    """

    type: Literal["DatatableColumnHint"] = "DatatableColumnHint"

    name: str
    """The name of the column."""

    item_type: str
    """The type of items in the column."""

    minimum: Primitive | None = None
    """The minimum value in the column."""

    maximum: Primitive | None = None
    """The maximum value in the column."""

    nulls: int | None = None
    """The number of `Null` values in the column."""


@dataclass(kw_only=True, repr=False)
class DatatableHint(Entity):
    """
    A hint to the structure of a table of data.
    """

    type: Literal["DatatableHint"] = "DatatableHint"

    rows: int
    """The number of rows of data."""

    columns: list[DatatableColumnHint]
    """A hint for each column of data."""


@dataclass(kw_only=True, repr=False)
class Date(Entity):
    """
    A calendar date encoded as a ISO 8601 string.
    """

    type: Literal["Date"] = "Date"

    value: str
    """The date as an ISO 8601 string."""


@dataclass(kw_only=True, repr=False)
class DateTime(Entity):
    """
    A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
    """

    type: Literal["DateTime"] = "DateTime"

    value: str
    """The date as an ISO 8601 string."""


@dataclass(kw_only=True, repr=False)
class DateTimeValidator(Entity):
    """
    A validator specifying the constraints on a date-time.
    """

    type: Literal["DateTimeValidator"] = "DateTimeValidator"

    minimum: DateTime | None = None
    """The inclusive lower limit for a date-time."""

    maximum: DateTime | None = None
    """The inclusive upper limit for a date-time."""


@dataclass(kw_only=True, repr=False)
class DateValidator(Entity):
    """
    A validator specifying the constraints on a date.
    """

    type: Literal["DateValidator"] = "DateValidator"

    minimum: Date | None = None
    """The inclusive lower limit for a date."""

    maximum: Date | None = None
    """The inclusive upper limit for a date."""


@dataclass(kw_only=True, repr=False)
class DefinedTerm(Thing):
    """
    A word, name, acronym, phrase, etc. with a formal definition.
    """

    type: Literal["DefinedTerm"] = "DefinedTerm"

    term_code: str | None = None
    """A code that identifies this DefinedTerm within a DefinedTermSet"""


@dataclass(kw_only=True, repr=False)
class DeleteBlock(SuggestionBlock):
    """
    A suggestion to delete some block content.
    """

    type: Literal["DeleteBlock"] = "DeleteBlock"


@dataclass(kw_only=True, repr=False)
class DeleteInline(SuggestionInline):
    """
    A suggestion to delete some inline content.
    """

    type: Literal["DeleteInline"] = "DeleteInline"


@dataclass(kw_only=True, repr=False)
class Directory(Entity):
    """
    A directory on the file system.
    """

    type: Literal["Directory"] = "Directory"

    name: str
    """The name of the directory."""

    path: str
    """The path (absolute or relative) of the file on the file system."""

    parts: list[File | Directory]
    """The files and other directories within this directory."""


@dataclass(kw_only=True, repr=False)
class Duration(Entity):
    """
    A value that represents the difference between two timestamps.
    """

    type: Literal["Duration"] = "Duration"

    value: int
    """The time difference in `timeUnit`s."""

    time_unit: TimeUnit
    """The time unit that the `value` represents."""


@dataclass(kw_only=True, repr=False)
class DurationValidator(Entity):
    """
    A validator specifying the constraints on a duration.
    """

    type: Literal["DurationValidator"] = "DurationValidator"

    time_units: list[TimeUnit] | None = None
    """The time units that the duration can have."""

    minimum: Duration | None = None
    """The inclusive lower limit for a duration."""

    maximum: Duration | None = None
    """The inclusive upper limit for a duration."""


@dataclass(kw_only=True, repr=False)
class Emphasis(Mark):
    """
    Emphasized content.
    """

    type: Literal["Emphasis"] = "Emphasis"


@dataclass(kw_only=True, repr=False)
class EnumValidator(Entity):
    """
    A schema specifying that a node must be one of several values.
    """

    type: Literal["EnumValidator"] = "EnumValidator"

    values: list[Node]
    """A node is valid if it is equal to any of these values."""


@dataclass(kw_only=True, repr=False)
class Enumeration(Thing):
    """
    Lists or enumerations, for example, a list of cuisines or music genres, etc.
    """

    type: Literal["Enumeration"] = "Enumeration"


@dataclass(kw_only=True, repr=False)
class ExecutionDependant(Entity):
    """
    A downstream execution dependant of a node.
    """

    type: Literal["ExecutionDependant"] = "ExecutionDependant"

    dependant_relation: ExecutionDependantRelation
    """The relation to the dependant."""

    dependant_node: ExecutionDependantNode
    """The node that is the dependant."""

    code_location: CodeLocation | None = None
    """The location that the dependant is defined."""


@dataclass(kw_only=True, repr=False)
class ExecutionDependency(Entity):
    """
    An upstream execution dependency of a node.
    """

    type: Literal["ExecutionDependency"] = "ExecutionDependency"

    dependency_relation: ExecutionDependencyRelation
    """The relation to the dependency."""

    dependency_node: ExecutionDependencyNode
    """The node that is the dependency."""

    code_location: CodeLocation | None = None
    """The location that the dependency is defined."""


@dataclass(kw_only=True, repr=False)
class ExecutionMessage(Entity):
    """
    An error, warning or log message generated during execution.
    """

    type: Literal["ExecutionMessage"] = "ExecutionMessage"

    level: MessageLevel
    """The severity level of the message."""

    message: str
    """The text of the message."""

    error_type: str | None = None
    """The type of error e.g. "SyntaxError", "ZeroDivisionError"."""

    code_location: CodeLocation | None = None
    """The location that the error occurred or other message emanated from."""

    stack_trace: str | None = None
    """Stack trace leading up to the error."""


@dataclass(kw_only=True, repr=False)
class ExecutionTag(Entity):
    """
    A tag on code that affects its execution.
    """

    type: Literal["ExecutionTag"] = "ExecutionTag"

    name: str
    """The name of the tag"""

    value: str
    """The value of the tag"""

    is_global: bool
    """Whether the tag is global to the document"""


@dataclass(kw_only=True, repr=False)
class Figure(CreativeWork):
    """
    Encapsulates one or more images, videos, tables, etc, and provides captions and labels for them.
    """

    type: Literal["Figure"] = "Figure"

    content: list[Block]
    """The content of the figure."""

    label: str | None = None
    """A short label for the figure."""

    caption: list[Block] | None = None
    """A caption for the figure."""


@dataclass(kw_only=True, repr=False)
class File(Entity):
    """
    A file on the file system.
    """

    type: Literal["File"] = "File"

    name: str
    """The name of the file."""

    path: str
    """The path (absolute or relative) of the file on the file system"""

    media_type: str | None = None
    """IANA media type (MIME type)."""


@dataclass(kw_only=True, repr=False)
class ForBlock(CodeExecutable):
    """
    Repeat a block content for each item in an array.
    """

    type: Literal["ForBlock"] = "ForBlock"

    variable: str
    """The name to give to the variable representing each item in the iterated array"""

    content: list[Block]
    """The content to repeat for each item"""

    otherwise: list[Block] | None = None
    """The content to render if there are no items"""

    iterations: list[Section] | None = None
    """The content repeated for each iteration"""


@dataclass(kw_only=True, repr=False)
class Form(Executable):
    """
    A form to batch updates in document parameters.
    """

    type: Literal["Form"] = "Form"

    content: list[Block]
    """The content within the form, usually containing at least one `Parameter`."""

    derive_from: str | None = None
    """The dotted path to the object (e.g a database table) that the form should be derived from"""

    derive_action: FormDeriveAction | None = None
    """The action (create, update or delete) to derive for the form"""

    derive_item: int | str | None = None
    """An identifier for the item to be the target of Update or Delete actions"""


@dataclass(kw_only=True, repr=False)
class Function(Entity):
    """
    A function with a name, which might take Parameters and return a value of a certain type.
    """

    type: Literal["Function"] = "Function"

    name: str
    """The name of the function."""

    parameters: list[Parameter]
    """The parameters of the function."""

    returns: Validator | None = None
    """The return type of the function."""


@dataclass(kw_only=True, repr=False)
class Heading(Entity):
    """
    A heading.
    """

    type: Literal["Heading"] = "Heading"

    level: int = 0
    """The level of the heading."""

    content: list[Inline]
    """Content of the heading."""

    authors: list[Author] | None = None
    """The authors of the heading."""


@dataclass(kw_only=True, repr=False)
class IfBlock(Executable):
    """
    Show and execute alternative content conditional upon an executed expression.
    """

    type: Literal["IfBlock"] = "IfBlock"

    clauses: list[IfBlockClause]
    """The clauses making up the `IfBlock` node"""

    authors: list[Author] | None = None
    """The authors of the if block."""


@dataclass(kw_only=True, repr=False)
class IfBlockClause(CodeExecutable):
    """
    A clause within an `IfBlock` node.
    """

    type: Literal["IfBlockClause"] = "IfBlockClause"

    is_active: bool | None = None
    """Whether this clause is the active clause in the parent `IfBlock` node"""

    content: list[Block]
    """The content to render if the result is truthy"""


@dataclass(kw_only=True, repr=False)
class ImageObject(MediaObject):
    """
    An image file.
    """

    type: Literal["ImageObject"] = "ImageObject"

    caption: list[Inline] | None = None
    """The caption for this image."""

    thumbnail: ImageObject | None = None
    """Thumbnail image of this image."""


@dataclass(kw_only=True, repr=False)
class InsertBlock(SuggestionBlock):
    """
    A suggestion to insert some block content.
    """

    type: Literal["InsertBlock"] = "InsertBlock"


@dataclass(kw_only=True, repr=False)
class InsertInline(SuggestionInline):
    """
    A suggestion to insert some inline content.
    """

    type: Literal["InsertInline"] = "InsertInline"


@dataclass(kw_only=True, repr=False)
class InstructionBlock(Instruction):
    """
    An instruction to edit some block content.
    """

    type: Literal["InstructionBlock"] = "InstructionBlock"

    content: list[Block] | None = None
    """The content to which the instruction applies."""

    suggestion: SuggestionBlockType | None = None
    """A suggestion for the instruction"""


@dataclass(kw_only=True, repr=False)
class InstructionInline(Instruction):
    """
    An instruction to edit some inline content.
    """

    type: Literal["InstructionInline"] = "InstructionInline"

    content: list[Inline] | None = None
    """The content to which the instruction applies."""

    suggestion: SuggestionInlineType | None = None
    """A suggestion for the instruction"""


@dataclass(kw_only=True, repr=False)
class InstructionMessage(Entity):
    """
    A message within an `Instruction`.
    """

    type: Literal["InstructionMessage"] = "InstructionMessage"

    parts: list[MessagePart]
    """Parts of the message."""

    content: list[Block] | None = None
    """Content of the message."""

    authors: list[Person | Organization | SoftwareApplication] | None = None
    """The authors of the message."""

    level: MessageLevel | None = None
    """The severity level of the message."""


@dataclass(kw_only=True, repr=False)
class IntegerValidator(NumberValidator):
    """
    A validator specifying the constraints on an integer node.
    """

    type: Literal["IntegerValidator"] = "IntegerValidator"


@dataclass(kw_only=True, repr=False)
class Link(Entity):
    """
    A hyperlink to other pages, sections within the same document, resources, or any URL.
    """

    type: Literal["Link"] = "Link"

    content: list[Inline]
    """The textual content of the link."""

    target: str
    """The target of the link."""

    title: str | None = None
    """A title for the link."""

    rel: str | None = None
    """The relation between the target and the current thing."""


@dataclass(kw_only=True, repr=False)
class List(Entity):
    """
    A list of items.
    """

    type: Literal["List"] = "List"

    items: list[ListItem]
    """The items in the list."""

    order: ListOrder
    """The ordering of the list."""

    authors: list[Author] | None = None
    """The authors of the list."""


@dataclass(kw_only=True, repr=False)
class ListItem(Thing):
    """
    A single item in a list.
    """

    type: Literal["ListItem"] = "ListItem"

    content: list[Block]
    """The content of the list item."""

    item: Node | None = None
    """The item represented by this list item."""

    is_checked: bool | None = None
    """A flag to indicate if this list item is checked."""

    position: int | None = None
    """The position of the item in a series or sequence of items."""


@dataclass(kw_only=True, repr=False)
class MathBlock(Math):
    """
    A block of math, e.g an equation, to be treated as block content.
    """

    type: Literal["MathBlock"] = "MathBlock"

    label: str | None = None
    """A short label for the math block."""


@dataclass(kw_only=True, repr=False)
class MathInline(Math):
    """
    A fragment of math, e.g a variable name, to be treated as inline content.
    """

    type: Literal["MathInline"] = "MathInline"


@dataclass(kw_only=True, repr=False)
class ModifyBlock(SuggestionBlock):
    """
    A suggestion to modify some block content.
    """

    type: Literal["ModifyBlock"] = "ModifyBlock"

    operations: list[ModifyOperation]
    """The operations to be applied to the nodes."""


@dataclass(kw_only=True, repr=False)
class ModifyInline(SuggestionInline):
    """
    A suggestion to modify some inline content.
    """

    type: Literal["ModifyInline"] = "ModifyInline"

    operations: list[ModifyOperation]
    """The operations to be applied to the nodes."""


@dataclass(kw_only=True, repr=False)
class ModifyOperation(Entity):
    """
    An operation that is part of a suggestion to modify the property of a node.
    """

    type: Literal["ModifyOperation"] = "ModifyOperation"

    target: str
    """The target property of each node to be modified."""

    value: StringPatch | Primitive
    """The new value, or string patch, to apply to the target property."""


@dataclass(kw_only=True, repr=False)
class MonetaryGrant(Grant):
    """
    A monetary grant.
    """

    type: Literal["MonetaryGrant"] = "MonetaryGrant"

    amounts: float | None = None
    """The amount of money."""

    funders: list[Person | Organization] | None = None
    """A person or organization that supports (sponsors) something through some kind of financial contribution."""


@dataclass(kw_only=True, repr=False)
class Note(Entity):
    """
    Additional content which is not part of the main content of a document.
    """

    type: Literal["Note"] = "Note"

    note_type: NoteType
    """Determines where the note content is displayed within the document."""

    content: list[Block]
    """Content of the note, usually a paragraph."""


@dataclass(kw_only=True, repr=False)
class ObjectHint(Entity):
    """
    A hint to the structure of an `Object`.
    """

    type: Literal["ObjectHint"] = "ObjectHint"

    length: int
    """The number of entries in the object."""

    keys: list[str]
    """The keys of the object's entries."""

    values: list[Hint]
    """Hints to the values of the object's entries."""


@dataclass(kw_only=True, repr=False)
class Organization(Thing):
    """
    An organization such as a school, NGO, corporation, club, etc.
    """

    type: Literal["Organization"] = "Organization"

    address: PostalAddress | str | None = None
    """Postal address for the organization."""

    brands: list[Brand] | None = None
    """Brands that the organization is connected with."""

    contact_points: list[ContactPoint] | None = None
    """Correspondence/Contact points for the organization."""

    departments: list[Organization] | None = None
    """Departments within the organization. For example, Department of Computer Science, Research & Development etc."""

    funders: list[Person | Organization] | None = None
    """Organization(s) or person(s) funding the organization."""

    legal_name: str | None = None
    """The official name of the organization, e.g. the registered company name."""

    logo: ImageObject | None = None
    """The logo of the organization."""

    members: list[Person | Organization] | None = None
    """Person(s) or organization(s) who are members of this organization."""

    parent_organization: Organization | None = None
    """Entity that the Organization is a part of. For example, parentOrganization to a department is a university."""


@dataclass(kw_only=True, repr=False)
class Paragraph(Entity):
    """
    A paragraph.
    """

    type: Literal["Paragraph"] = "Paragraph"

    content: list[Inline]
    """The contents of the paragraph."""

    authors: list[Author] | None = None
    """The authors of the paragraph."""


@dataclass(kw_only=True, repr=False)
class Periodical(CreativeWork):
    """
    A periodical publication.
    """

    type: Literal["Periodical"] = "Periodical"

    date_start: Date | None = None
    """The date this Periodical was first published."""

    date_end: Date | None = None
    """The date this Periodical ceased publication."""

    issns: list[str] | None = None
    """The International Standard Serial Number(s) (ISSN) that identifies this serial publication."""


@dataclass(kw_only=True, repr=False)
class Person(Thing):
    """
    A person (alive, dead, undead, or fictional).
    """

    type: Literal["Person"] = "Person"

    address: PostalAddress | str | None = None
    """Postal address for the person."""

    affiliations: list[Organization] | None = None
    """Organizations that the person is affiliated with."""

    emails: list[str] | None = None
    """Email addresses for the person."""

    family_names: list[str] | None = None
    """Family name. In the U.S., the last name of a person."""

    funders: list[Person | Organization] | None = None
    """A person or organization that supports (sponsors) something through some kind of financial contribution."""

    given_names: list[str] | None = None
    """Given name. In the U.S., the first name of a person."""

    honorific_prefix: str | None = None
    """An honorific prefix preceding a person's name such as Dr/Mrs/Mr."""

    honorific_suffix: str | None = None
    """An honorific suffix after a person's name such as MD/PhD/MSCSW."""

    job_title: str | None = None
    """The job title of the person (for example, Financial Manager)."""

    member_of: list[Organization] | None = None
    """An organization (or program membership) to which this person belongs."""

    telephone_numbers: list[str] | None = None
    """Telephone numbers for the person."""


@dataclass(kw_only=True, repr=False)
class PostalAddress(ContactPoint):
    """
    A physical mailing address.
    """

    type: Literal["PostalAddress"] = "PostalAddress"

    street_address: str | None = None
    """The street address."""

    post_office_box_number: str | None = None
    """The post office box number."""

    address_locality: str | None = None
    """The locality in which the street address is, and which is in the region."""

    address_region: str | None = None
    """The region in which the locality is, and which is in the country."""

    postal_code: str | None = None
    """The postal code."""

    address_country: str | None = None
    """The country."""


@dataclass(kw_only=True, repr=False)
class Product(Thing):
    """
    Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
    """

    type: Literal["Product"] = "Product"

    brands: list[Brand] | None = None
    """Brands that the product is labelled with."""

    logo: ImageObject | None = None
    """The logo of the product."""

    product_id: str | None = None
    """Product identification code."""


@dataclass(kw_only=True, repr=False)
class PropertyValue(Thing):
    """
    A property-value pair.
    """

    type: Literal["PropertyValue"] = "PropertyValue"

    property_id: str | None = None
    """A commonly used identifier for the characteristic represented by the property."""

    value: Primitive
    """The value of the property."""


@dataclass(kw_only=True, repr=False)
class PublicationIssue(CreativeWork):
    """
    A part of a successively published publication such as a periodical or publication volume, often numbered.
    """

    type: Literal["PublicationIssue"] = "PublicationIssue"

    issue_number: int | str | None = None
    """Identifies the issue of publication; for example, "iii" or "2"."""

    page_start: int | str | None = None
    """The page on which the issue starts; for example "135" or "xiii"."""

    page_end: int | str | None = None
    """The page on which the issue ends; for example "138" or "xvi"."""

    pagination: str | None = None
    """Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55"."""


@dataclass(kw_only=True, repr=False)
class PublicationVolume(CreativeWork):
    """
    A part of a successively published publication such as a periodical or multi-volume work.
    """

    type: Literal["PublicationVolume"] = "PublicationVolume"

    page_start: int | str | None = None
    """The page on which the volume starts; for example "135" or "xiii"."""

    page_end: int | str | None = None
    """The page on which the volume ends; for example "138" or "xvi"."""

    pagination: str | None = None
    """Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55"."""

    volume_number: int | str | None = None
    """Identifies the volume of publication or multi-part work; for example, "iii" or "2"."""


@dataclass(kw_only=True, repr=False)
class QuoteBlock(Entity):
    """
    A section quoted from somewhere else.
    """

    type: Literal["QuoteBlock"] = "QuoteBlock"

    cite: Cite | Text | None = None
    """The source of the quote."""

    content: list[Block]
    """The content of the quote."""

    authors: list[Author] | None = None
    """The authors of the quote."""


@dataclass(kw_only=True, repr=False)
class QuoteInline(Mark):
    """
    Inline, quoted content.
    """

    type: Literal["QuoteInline"] = "QuoteInline"

    cite: Cite | Text | None = None
    """The source of the quote."""


@dataclass(kw_only=True, repr=False)
class ReplaceBlock(SuggestionBlock):
    """
    A suggestion to replace some block content with new block content.
    """

    type: Literal["ReplaceBlock"] = "ReplaceBlock"

    replacement: list[Block]
    """The new replacement block content."""


@dataclass(kw_only=True, repr=False)
class ReplaceInline(SuggestionInline):
    """
    A suggestion to replace some inline content with new inline content.
    """

    type: Literal["ReplaceInline"] = "ReplaceInline"

    replacement: list[Inline]
    """The new replacement inline content."""


@dataclass(kw_only=True, repr=False)
class Review(CreativeWork):
    """
    A review of an item, e.g of an `Article` or `SoftwareApplication`.
    """

    type: Literal["Review"] = "Review"

    item_reviewed: Thing | None = None
    """The item that is being reviewed."""

    review_aspect: str | None = None
    """The part or facet of the item that is being reviewed."""


@dataclass(kw_only=True, repr=False)
class Section(Entity):
    """
    A section of a document.
    """

    type: Literal["Section"] = "Section"

    content: list[Block]
    """The content within the section."""

    section_type: SectionType | None = None
    """The type of section."""


@dataclass(kw_only=True, repr=False)
class SoftwareApplication(CreativeWork):
    """
    A software application.
    """

    type: Literal["SoftwareApplication"] = "SoftwareApplication"

    software_requirements: list[SoftwareApplication] | None = None
    """Requirements for application, including shared libraries that are not included in the application distribution."""

    software_version: str | None = None
    """Version of the software."""

    operating_system: str | None = None
    """Operating systems supported (e.g. Windows 7, OS X 10.6)."""


@dataclass(kw_only=True, repr=False)
class SoftwareSourceCode(CreativeWork):
    """
    Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
    """

    type: Literal["SoftwareSourceCode"] = "SoftwareSourceCode"

    programming_language: str
    """The computer programming language."""

    code_repository: str | None = None
    """Link to the repository where the un-compiled, human readable code and related code is located."""

    code_sample_type: str | None = None
    """What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template."""

    runtime_platform: list[str] | None = None
    """Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0)."""

    software_requirements: list[SoftwareSourceCode | SoftwareApplication | str] | None = None
    """Dependency requirements for the software."""

    target_products: list[SoftwareApplication] | None = None
    """Target operating system or product to which the code applies."""


@dataclass(kw_only=True, repr=False)
class Strikeout(Mark):
    """
    Content that is marked as struck out.
    """

    type: Literal["Strikeout"] = "Strikeout"


@dataclass(kw_only=True, repr=False)
class StringHint(Entity):
    """
    A hint to the structure of an `String`.
    """

    type: Literal["StringHint"] = "StringHint"

    chars: int
    """The number of characters in the string."""


@dataclass(kw_only=True, repr=False)
class StringOperation(Entity):
    """
    An operation that modifies a string.
    """

    type: Literal["StringOperation"] = "StringOperation"

    start_position: UnsignedInteger
    """The start position in the string of the operation."""

    end_position: UnsignedInteger | None = None
    """The end position in the string of the operation."""

    value: str | None = None
    """The string value to insert or use as the replacement."""


@dataclass(kw_only=True, repr=False)
class StringPatch(Entity):
    """
    An set of operations to modify a string.
    """

    type: Literal["StringPatch"] = "StringPatch"

    operations: list[StringOperation]
    """The operations to be applied to the string."""


@dataclass(kw_only=True, repr=False)
class StringValidator(Entity):
    """
    A schema specifying constraints on a string node.
    """

    type: Literal["StringValidator"] = "StringValidator"

    min_length: int | None = None
    """The minimum length for a string node."""

    max_length: int | None = None
    """The maximum length for a string node."""

    pattern: str | None = None
    """A regular expression that a string node must match."""


@dataclass(kw_only=True, repr=False)
class Strong(Mark):
    """
    Strongly emphasized content.
    """

    type: Literal["Strong"] = "Strong"


@dataclass(kw_only=True, repr=False)
class StyledBlock(Styled):
    """
    Styled block content.
    """

    type: Literal["StyledBlock"] = "StyledBlock"

    content: list[Block]
    """The content within the styled block"""


@dataclass(kw_only=True, repr=False)
class StyledInline(Styled):
    """
    Styled inline content.
    """

    type: Literal["StyledInline"] = "StyledInline"

    content: list[Inline]
    """The content within the span."""


@dataclass(kw_only=True, repr=False)
class Subscript(Mark):
    """
    Subscripted content.
    """

    type: Literal["Subscript"] = "Subscript"


@dataclass(kw_only=True, repr=False)
class Superscript(Mark):
    """
    Superscripted content.
    """

    type: Literal["Superscript"] = "Superscript"


@dataclass(kw_only=True, repr=False)
class Table(CreativeWork):
    """
    A table.
    """

    type: Literal["Table"] = "Table"

    label: str | None = None
    """A short label for the table."""

    caption: list[Block] | None = None
    """A caption for the table."""

    rows: list[TableRow]
    """Rows of cells in the table."""

    notes: list[Block] | None = None
    """Notes for the table."""


@dataclass(kw_only=True, repr=False)
class TableCell(Entity):
    """
    A cell within a `Table`.
    """

    type: Literal["TableCell"] = "TableCell"

    cell_type: TableCellType | None = None
    """The type of cell."""

    name: str | None = None
    """The name of the cell."""

    column_span: int | None = None
    """How many columns the cell extends."""

    row_span: int | None = None
    """How many columns the cell extends."""

    content: list[Block]
    """Contents of the table cell."""


@dataclass(kw_only=True, repr=False)
class TableRow(Entity):
    """
    A row within a Table.
    """

    type: Literal["TableRow"] = "TableRow"

    cells: list[TableCell]
    """An array of cells in the row."""

    row_type: TableRowType | None = None
    """The type of row."""


@dataclass(kw_only=True, repr=False)
class Text(Entity):
    """
    Textual content.
    """

    type: Literal["Text"] = "Text"

    value: Cord
    """The value of the text content"""


@dataclass(kw_only=True, repr=False)
class ThematicBreak(Entity):
    """
    A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
    """

    type: Literal["ThematicBreak"] = "ThematicBreak"


@dataclass(kw_only=True, repr=False)
class Time(Entity):
    """
    A point in time recurring on multiple days.
    """

    type: Literal["Time"] = "Time"

    value: str
    """The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`."""


@dataclass(kw_only=True, repr=False)
class TimeValidator(Entity):
    """
    A validator specifying the constraints on a time.
    """

    type: Literal["TimeValidator"] = "TimeValidator"

    minimum: Time | None = None
    """The inclusive lower limit for a time."""

    maximum: Time | None = None
    """The inclusive upper limit for a time."""


@dataclass(kw_only=True, repr=False)
class Timestamp(Entity):
    """
    A value that represents a point in time.
    """

    type: Literal["Timestamp"] = "Timestamp"

    value: int
    """The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z)."""

    time_unit: TimeUnit
    """The time unit that the `value` represents."""


@dataclass(kw_only=True, repr=False)
class TimestampValidator(Entity):
    """
    A validator specifying the constraints on a timestamp.
    """

    type: Literal["TimestampValidator"] = "TimestampValidator"

    time_units: list[TimeUnit] | None = None
    """The time units that the timestamp can have."""

    minimum: Timestamp | None = None
    """The inclusive lower limit for a timestamp."""

    maximum: Timestamp | None = None
    """The inclusive upper limit for a timestamp."""


@dataclass(kw_only=True, repr=False)
class TupleValidator(Entity):
    """
    A validator specifying constraints on an array of heterogeneous items.
    """

    type: Literal["TupleValidator"] = "TupleValidator"

    items: list[Validator] | None = None
    """An array of validators specifying the constraints on each successive item in the array."""


@dataclass(kw_only=True, repr=False)
class Underline(Mark):
    """
    Inline text that is underlined.
    """

    type: Literal["Underline"] = "Underline"


@dataclass(kw_only=True, repr=False)
class Unknown(Entity):
    """
    A type to indicate a value or or other type in unknown.
    """

    type: Literal["Unknown"] = "Unknown"


@dataclass(kw_only=True, repr=False)
class Variable(Entity):
    """
    A variable representing a name / value pair.
    """

    type: Literal["Variable"] = "Variable"

    name: str
    """The name of the variable."""

    programming_language: str | None = None
    """The programming language that the variable is defined in e.g. Python, JSON."""

    native_type: str | None = None
    """The native type of the variable e.g. `float`, `datetime.datetime`, `pandas.DataFrame`"""

    node_type: str | None = None
    """The Stencila node type of the variable e.g. `Number`, `DateTime`, `Datatable`."""

    value: Node | None = None
    """The value of the variable."""

    hint: Hint | None = None
    """A hint to the value and/or structure of the variable."""

    native_hint: str | None = None
    """A textual hint to the value and/or structure of the variable."""


@dataclass(kw_only=True, repr=False)
class VideoObject(MediaObject):
    """
    A video file.
    """

    type: Literal["VideoObject"] = "VideoObject"

    caption: list[Inline] | None = None
    """The caption for this video recording."""

    thumbnail: ImageObject | None = None
    """Thumbnail image of this video recording."""

    transcript: str | None = None
    """The transcript of this video recording."""

Author = Union[
    Person,
    Organization,
    SoftwareApplication,
    AuthorRole,
]
"""
Union type for things that can be an author of a `CreativeWork` or other type.
"""


Block = Union[
    Admonition,
    CallBlock,
    Claim,
    CodeBlock,
    CodeChunk,
    DeleteBlock,
    Figure,
    ForBlock,
    Form,
    Heading,
    IfBlock,
    IncludeBlock,
    InsertBlock,
    InstructionBlock,
    List,
    MathBlock,
    ModifyBlock,
    Paragraph,
    QuoteBlock,
    ReplaceBlock,
    Section,
    StyledBlock,
    Table,
    ThematicBreak,
]
"""
Union type in block content node types.
"""


CreativeWorkType = Union[
    Article,
    AudioObject,
    Claim,
    Collection,
    Comment,
    Datatable,
    Figure,
    ImageObject,
    MediaObject,
    Periodical,
    PublicationIssue,
    PublicationVolume,
    Review,
    SoftwareApplication,
    SoftwareSourceCode,
    Table,
    VideoObject,
]
"""
Union type for all types that are descended from `CreativeWork`
"""


ExecutionDependantNode = Union[
    Button,
    CallBlock,
    CodeChunk,
    CodeExpression,
    File,
    Function,
    Parameter,
    StyledBlock,
    StyledInline,
    Variable,
]
"""
Node types that can be execution dependencies.
"""


ExecutionDependencyNode = Union[
    Button,
    CodeChunk,
    File,
    Parameter,
    SoftwareSourceCode,
    Variable,
]
"""
Node types that can be execution dependencies.
"""


Hint = Union[
    ArrayHint,
    DatatableHint,
    Function,
    ObjectHint,
    StringHint,
    Unknown,
    bool,
    int,
    float,
]
"""
Union type for hints of the value and/or structure of data.
"""


Inline = Union[
    AudioObject,
    Button,
    Cite,
    CiteGroup,
    CodeExpression,
    CodeInline,
    Date,
    DateTime,
    DeleteInline,
    Duration,
    Emphasis,
    ImageObject,
    InsertInline,
    InstructionInline,
    Link,
    MathInline,
    MediaObject,
    ModifyInline,
    Note,
    Parameter,
    QuoteInline,
    ReplaceInline,
    StyledInline,
    Strikeout,
    Strong,
    Subscript,
    Superscript,
    Text,
    Time,
    Timestamp,
    Underline,
    VideoObject,
    None,
    bool,
    int,
    UnsignedInteger,
    float,
]
"""
Union type for valid inline content.
"""


MessagePart = Union[
    Text,
    ImageObject,
    AudioObject,
    VideoObject,
]
"""
A union type for a part of a message.
"""


Node = Union[
    None,
    bool,
    int,
    UnsignedInteger,
    float,
    str,
    Cord,
    Array,
    Admonition,
    ArrayHint,
    ArrayValidator,
    Article,
    AudioObject,
    AuthorRole,
    BooleanValidator,
    Brand,
    Button,
    CallArgument,
    CallBlock,
    Cite,
    CiteGroup,
    Claim,
    CodeBlock,
    CodeChunk,
    CodeExpression,
    CodeInline,
    CodeLocation,
    Collection,
    Comment,
    CompilationDigest,
    CompilationMessage,
    ConstantValidator,
    ContactPoint,
    CreativeWork,
    Datatable,
    DatatableColumn,
    DatatableColumnHint,
    DatatableHint,
    Date,
    DateTime,
    DateTimeValidator,
    DateValidator,
    DefinedTerm,
    DeleteBlock,
    DeleteInline,
    Directory,
    Duration,
    DurationValidator,
    Emphasis,
    EnumValidator,
    Enumeration,
    ExecutionDependant,
    ExecutionDependency,
    ExecutionMessage,
    ExecutionTag,
    Figure,
    File,
    ForBlock,
    Form,
    Function,
    Grant,
    Heading,
    IfBlock,
    IfBlockClause,
    ImageObject,
    IncludeBlock,
    InsertBlock,
    InsertInline,
    InstructionBlock,
    InstructionInline,
    InstructionMessage,
    IntegerValidator,
    Link,
    List,
    ListItem,
    MathBlock,
    MathInline,
    MediaObject,
    ModifyBlock,
    ModifyInline,
    ModifyOperation,
    MonetaryGrant,
    Note,
    NumberValidator,
    ObjectHint,
    Organization,
    Paragraph,
    Parameter,
    Periodical,
    Person,
    PostalAddress,
    Product,
    PropertyValue,
    PublicationIssue,
    PublicationVolume,
    QuoteBlock,
    QuoteInline,
    ReplaceBlock,
    ReplaceInline,
    Review,
    Section,
    SoftwareApplication,
    SoftwareSourceCode,
    Strikeout,
    StringHint,
    StringOperation,
    StringPatch,
    StringValidator,
    Strong,
    StyledBlock,
    StyledInline,
    Subscript,
    Superscript,
    Table,
    TableCell,
    TableRow,
    Text,
    ThematicBreak,
    Thing,
    Time,
    TimeValidator,
    Timestamp,
    TimestampValidator,
    TupleValidator,
    Underline,
    Unknown,
    Variable,
    VideoObject,
    Object,
]
"""
Union type for all types in this schema, including primitives and entities
"""


SuggestionBlockType = Union[
    DeleteBlock,
    InsertBlock,
    ModifyBlock,
    ReplaceBlock,
]
"""
Union type for all types that are descended from `SuggestionBlock`
"""


SuggestionInlineType = Union[
    DeleteInline,
    InsertInline,
    ModifyInline,
    ReplaceInline,
]
"""
Union type for all types that are descended from `SuggestionInline`
"""


ThingType = Union[
    AdmonitionType,
    Article,
    AudioObject,
    AuthorRoleName,
    AutomaticExecution,
    Brand,
    CitationIntent,
    CitationMode,
    Claim,
    ClaimType,
    Collection,
    Comment,
    ContactPoint,
    CreativeWork,
    Datatable,
    DefinedTerm,
    Enumeration,
    ExecutionDependantRelation,
    ExecutionDependencyRelation,
    ExecutionRequired,
    ExecutionStatus,
    Figure,
    FormDeriveAction,
    Grant,
    ImageObject,
    LabelType,
    ListItem,
    ListOrder,
    MediaObject,
    MessageLevel,
    MonetaryGrant,
    NoteType,
    Organization,
    Periodical,
    Person,
    PostalAddress,
    Product,
    PropertyValue,
    PublicationIssue,
    PublicationVolume,
    Review,
    SectionType,
    SoftwareApplication,
    SoftwareSourceCode,
    SuggestionStatus,
    Table,
    TableCellType,
    TableRowType,
    TimeUnit,
    VideoObject,
]
"""
Union type for all types that are descended from `Thing`
"""


Validator = Union[
    ArrayValidator,
    BooleanValidator,
    ConstantValidator,
    DateTimeValidator,
    DateValidator,
    DurationValidator,
    EnumValidator,
    IntegerValidator,
    NumberValidator,
    StringValidator,
    TimeValidator,
    TimestampValidator,
    TupleValidator,
]
"""
Union type for validators.
"""
