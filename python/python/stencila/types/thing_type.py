# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

AdmonitionType = ForwardRef("AdmonitionType")
Article = ForwardRef("Article")
AudioObject = ForwardRef("AudioObject")
AutomaticExecution = ForwardRef("AutomaticExecution")
Brand = ForwardRef("Brand")
CitationIntent = ForwardRef("CitationIntent")
CitationMode = ForwardRef("CitationMode")
Claim = ForwardRef("Claim")
ClaimType = ForwardRef("ClaimType")
Collection = ForwardRef("Collection")
Comment = ForwardRef("Comment")
ContactPoint = ForwardRef("ContactPoint")
CreativeWork = ForwardRef("CreativeWork")
Datatable = ForwardRef("Datatable")
DatatableColumn = ForwardRef("DatatableColumn")
DefinedTerm = ForwardRef("DefinedTerm")
Directory = ForwardRef("Directory")
Enumeration = ForwardRef("Enumeration")
ExecutionDependantRelation = ForwardRef("ExecutionDependantRelation")
ExecutionDependencyRelation = ForwardRef("ExecutionDependencyRelation")
ExecutionRequired = ForwardRef("ExecutionRequired")
ExecutionStatus = ForwardRef("ExecutionStatus")
Figure = ForwardRef("Figure")
File = ForwardRef("File")
FormDeriveAction = ForwardRef("FormDeriveAction")
Grant = ForwardRef("Grant")
ImageObject = ForwardRef("ImageObject")
ListItem = ForwardRef("ListItem")
ListOrder = ForwardRef("ListOrder")
MediaObject = ForwardRef("MediaObject")
MonetaryGrant = ForwardRef("MonetaryGrant")
NoteType = ForwardRef("NoteType")
Organization = ForwardRef("Organization")
Periodical = ForwardRef("Periodical")
Person = ForwardRef("Person")
PostalAddress = ForwardRef("PostalAddress")
Product = ForwardRef("Product")
PropertyValue = ForwardRef("PropertyValue")
PublicationIssue = ForwardRef("PublicationIssue")
PublicationVolume = ForwardRef("PublicationVolume")
Review = ForwardRef("Review")
SectionType = ForwardRef("SectionType")
SoftwareApplication = ForwardRef("SoftwareApplication")
SoftwareSourceCode = ForwardRef("SoftwareSourceCode")
Table = ForwardRef("Table")
TableCellType = ForwardRef("TableCellType")
TableRowType = ForwardRef("TableRowType")
TimeUnit = ForwardRef("TimeUnit")
VideoObject = ForwardRef("VideoObject")


ThingType = Union[
    AdmonitionType,
    Article,
    AudioObject,
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
    DatatableColumn,
    DefinedTerm,
    Directory,
    Enumeration,
    ExecutionDependantRelation,
    ExecutionDependencyRelation,
    ExecutionRequired,
    ExecutionStatus,
    Figure,
    File,
    FormDeriveAction,
    Grant,
    ImageObject,
    ListItem,
    ListOrder,
    MediaObject,
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
    Table,
    TableCellType,
    TableRowType,
    TimeUnit,
    VideoObject,
]
"""
Union type for all types that are descended from `Thing`
"""
