# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Admonition = ForwardRef("Admonition")
Array = ForwardRef("Array")
ArrayValidator = ForwardRef("ArrayValidator")
Article = ForwardRef("Article")
AudioObject = ForwardRef("AudioObject")
BooleanValidator = ForwardRef("BooleanValidator")
Brand = ForwardRef("Brand")
Button = ForwardRef("Button")
Call = ForwardRef("Call")
CallArgument = ForwardRef("CallArgument")
Cite = ForwardRef("Cite")
CiteGroup = ForwardRef("CiteGroup")
Claim = ForwardRef("Claim")
CodeBlock = ForwardRef("CodeBlock")
CodeChunk = ForwardRef("CodeChunk")
CodeExpression = ForwardRef("CodeExpression")
CodeInline = ForwardRef("CodeInline")
CodeLocation = ForwardRef("CodeLocation")
Collection = ForwardRef("Collection")
Comment = ForwardRef("Comment")
CompilationDigest = ForwardRef("CompilationDigest")
CompilationError = ForwardRef("CompilationError")
ConstantValidator = ForwardRef("ConstantValidator")
ContactPoint = ForwardRef("ContactPoint")
Cord = ForwardRef("Cord")
CreativeWork = ForwardRef("CreativeWork")
Datatable = ForwardRef("Datatable")
DatatableColumn = ForwardRef("DatatableColumn")
Date = ForwardRef("Date")
DateTime = ForwardRef("DateTime")
DateTimeValidator = ForwardRef("DateTimeValidator")
DateValidator = ForwardRef("DateValidator")
DefinedTerm = ForwardRef("DefinedTerm")
Delete = ForwardRef("Delete")
Directory = ForwardRef("Directory")
Duration = ForwardRef("Duration")
DurationValidator = ForwardRef("DurationValidator")
Emphasis = ForwardRef("Emphasis")
EnumValidator = ForwardRef("EnumValidator")
Enumeration = ForwardRef("Enumeration")
ExecutionDependant = ForwardRef("ExecutionDependant")
ExecutionDependency = ForwardRef("ExecutionDependency")
ExecutionError = ForwardRef("ExecutionError")
ExecutionTag = ForwardRef("ExecutionTag")
Figure = ForwardRef("Figure")
File = ForwardRef("File")
For = ForwardRef("For")
Form = ForwardRef("Form")
Function = ForwardRef("Function")
Grant = ForwardRef("Grant")
Heading = ForwardRef("Heading")
If = ForwardRef("If")
IfClause = ForwardRef("IfClause")
ImageObject = ForwardRef("ImageObject")
Include = ForwardRef("Include")
Insert = ForwardRef("Insert")
IntegerValidator = ForwardRef("IntegerValidator")
Link = ForwardRef("Link")
List = ForwardRef("List")
ListItem = ForwardRef("ListItem")
MathBlock = ForwardRef("MathBlock")
MathInline = ForwardRef("MathInline")
MediaObject = ForwardRef("MediaObject")
MonetaryGrant = ForwardRef("MonetaryGrant")
Note = ForwardRef("Note")
NumberValidator = ForwardRef("NumberValidator")
Object = ForwardRef("Object")
Organization = ForwardRef("Organization")
Paragraph = ForwardRef("Paragraph")
Parameter = ForwardRef("Parameter")
Periodical = ForwardRef("Periodical")
Person = ForwardRef("Person")
PostalAddress = ForwardRef("PostalAddress")
Product = ForwardRef("Product")
PropertyValue = ForwardRef("PropertyValue")
PublicationIssue = ForwardRef("PublicationIssue")
PublicationVolume = ForwardRef("PublicationVolume")
QuoteBlock = ForwardRef("QuoteBlock")
QuoteInline = ForwardRef("QuoteInline")
Review = ForwardRef("Review")
Section = ForwardRef("Section")
SoftwareApplication = ForwardRef("SoftwareApplication")
SoftwareSourceCode = ForwardRef("SoftwareSourceCode")
Strikeout = ForwardRef("Strikeout")
StringValidator = ForwardRef("StringValidator")
Strong = ForwardRef("Strong")
StyledBlock = ForwardRef("StyledBlock")
StyledInline = ForwardRef("StyledInline")
Subscript = ForwardRef("Subscript")
Superscript = ForwardRef("Superscript")
Table = ForwardRef("Table")
TableCell = ForwardRef("TableCell")
TableRow = ForwardRef("TableRow")
Text = ForwardRef("Text")
ThematicBreak = ForwardRef("ThematicBreak")
Thing = ForwardRef("Thing")
Time = ForwardRef("Time")
TimeValidator = ForwardRef("TimeValidator")
Timestamp = ForwardRef("Timestamp")
TimestampValidator = ForwardRef("TimestampValidator")
TupleValidator = ForwardRef("TupleValidator")
Underline = ForwardRef("Underline")
UnsignedInteger = ForwardRef("UnsignedInteger")
Variable = ForwardRef("Variable")
VideoObject = ForwardRef("VideoObject")


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
    CodeExpression,
    CodeInline,
    CodeLocation,
    Collection,
    Comment,
    CompilationDigest,
    CompilationError,
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
    Delete,
    Directory,
    Duration,
    DurationValidator,
    Emphasis,
    EnumValidator,
    Enumeration,
    ExecutionDependant,
    ExecutionDependency,
    ExecutionError,
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
    Insert,
    IntegerValidator,
    Link,
    List,
    ListItem,
    MathBlock,
    MathInline,
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
    QuoteBlock,
    QuoteInline,
    Review,
    Section,
    SoftwareApplication,
    SoftwareSourceCode,
    Strikeout,
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
    Variable,
    VideoObject,
    Object,
]
"""
Union type for all types in this schema, including primitives and entities
"""
