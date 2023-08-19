# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .array import Array
from .array_validator import ArrayValidator
from .article import Article
from .audio_object import AudioObject
from .boolean_validator import BooleanValidator
from .brand import Brand
from .button import Button
from .call import Call
from .call_argument import CallArgument
from .cite import Cite
from .cite_group import CiteGroup
from .claim import Claim
from .code_block import CodeBlock
from .code_chunk import CodeChunk
from .code_error import CodeError
from .code_expression import CodeExpression
from .code_fragment import CodeFragment
from .collection import Collection
from .comment import Comment
from .constant_validator import ConstantValidator
from .contact_point import ContactPoint
from .creative_work import CreativeWork
from .datatable import Datatable
from .datatable_column import DatatableColumn
from .date import Date
from .date_time import DateTime
from .date_time_validator import DateTimeValidator
from .date_validator import DateValidator
from .defined_term import DefinedTerm
from .directory import Directory
from .division import Division
from .duration import Duration
from .duration_validator import DurationValidator
from .emphasis import Emphasis
from .enum_validator import EnumValidator
from .enumeration import Enumeration
from .execution_dependant import ExecutionDependant
from .execution_dependency import ExecutionDependency
from .execution_digest import ExecutionDigest
from .execution_tag import ExecutionTag
from .figure import Figure
from .file import File
from .for_ import For
from .form import Form
from .function import Function
from .grant import Grant
from .heading import Heading
from .if_ import If
from .if_clause import IfClause
from .image_object import ImageObject
from .include import Include
from .integer_validator import IntegerValidator
from .link import Link
from .list import List
from .list_item import ListItem
from .math_block import MathBlock
from .math_fragment import MathFragment
from .media_object import MediaObject
from .monetary_grant import MonetaryGrant
from .note import Note
from .null import Null
from .number_validator import NumberValidator
from .object import Object
from .organization import Organization
from .paragraph import Paragraph
from .parameter import Parameter
from .periodical import Periodical
from .person import Person
from .postal_address import PostalAddress
from .product import Product
from .property_value import PropertyValue
from .publication_issue import PublicationIssue
from .publication_volume import PublicationVolume
from .quote import Quote
from .quote_block import QuoteBlock
from .review import Review
from .software_application import SoftwareApplication
from .software_source_code import SoftwareSourceCode
from .span import Span
from .strikeout import Strikeout
from .string_validator import StringValidator
from .strong import Strong
from .subscript import Subscript
from .superscript import Superscript
from .table import Table
from .table_cell import TableCell
from .table_row import TableRow
from .text import Text
from .thematic_break import ThematicBreak
from .thing import Thing
from .time import Time
from .time_validator import TimeValidator
from .timestamp import Timestamp
from .timestamp_validator import TimestampValidator
from .tuple_validator import TupleValidator
from .underline import Underline
from .unsigned_integer import UnsignedInteger
from .variable import Variable
from .video_object import VideoObject


Node = Union[
    Null,
    bool,
    int,
    UnsignedInteger,
    float,
    str,
    Array,
    ArrayValidator,
    Article,
    AudioObject,
    BooleanValidator,
    Brand,
    Button,
    Call,
    CallArgument,
    Cite,
    CiteGroup,
    Claim,
    CodeBlock,
    CodeChunk,
    CodeError,
    CodeExpression,
    CodeFragment,
    Collection,
    Comment,
    ConstantValidator,
    ContactPoint,
    CreativeWork,
    Datatable,
    DatatableColumn,
    Date,
    DateTime,
    DateTimeValidator,
    DateValidator,
    DefinedTerm,
    Directory,
    Division,
    Duration,
    DurationValidator,
    Emphasis,
    EnumValidator,
    Enumeration,
    ExecutionDependant,
    ExecutionDependency,
    ExecutionDigest,
    ExecutionTag,
    Figure,
    File,
    For,
    Form,
    Function,
    Grant,
    Heading,
    If,
    IfClause,
    ImageObject,
    Include,
    IntegerValidator,
    Link,
    List,
    ListItem,
    MathBlock,
    MathFragment,
    MediaObject,
    MonetaryGrant,
    Note,
    NumberValidator,
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
    Quote,
    QuoteBlock,
    Review,
    SoftwareApplication,
    SoftwareSourceCode,
    Span,
    Strikeout,
    StringValidator,
    Strong,
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
    Variable,
    VideoObject,
    Object,
]
"""
Union type for all types in this schema, including primitives and entities
"""
