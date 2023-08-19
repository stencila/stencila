# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .article import Article
from .audio_object import AudioObject
from .brand import Brand
from .citation_intent import CitationIntent
from .citation_mode import CitationMode
from .claim import Claim
from .claim_type import ClaimType
from .collection import Collection
from .comment import Comment
from .contact_point import ContactPoint
from .creative_work import CreativeWork
from .datatable import Datatable
from .datatable_column import DatatableColumn
from .defined_term import DefinedTerm
from .directory import Directory
from .enumeration import Enumeration
from .execution_auto import ExecutionAuto
from .execution_dependant_relation import ExecutionDependantRelation
from .execution_dependency_relation import ExecutionDependencyRelation
from .execution_required import ExecutionRequired
from .execution_status import ExecutionStatus
from .figure import Figure
from .file import File
from .form_derive_action import FormDeriveAction
from .grant import Grant
from .image_object import ImageObject
from .list_item import ListItem
from .list_order import ListOrder
from .media_object import MediaObject
from .monetary_grant import MonetaryGrant
from .note_type import NoteType
from .organization import Organization
from .periodical import Periodical
from .person import Person
from .postal_address import PostalAddress
from .product import Product
from .property_value import PropertyValue
from .publication_issue import PublicationIssue
from .publication_volume import PublicationVolume
from .review import Review
from .software_application import SoftwareApplication
from .software_source_code import SoftwareSourceCode
from .table import Table
from .table_cell_type import TableCellType
from .table_row_type import TableRowType
from .time_unit import TimeUnit
from .video_object import VideoObject


ThingType = Union[
    Article,
    AudioObject,
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
    ExecutionAuto,
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
