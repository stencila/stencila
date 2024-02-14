# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Admonition = ForwardRef("Admonition")
Array = ForwardRef("Array")
ArrayHint = ForwardRef("ArrayHint")
ArrayValidator = ForwardRef("ArrayValidator")
Article = ForwardRef("Article")
AudioObject = ForwardRef("AudioObject")
AuthorRole = ForwardRef("AuthorRole")
BooleanValidator = ForwardRef("BooleanValidator")
Brand = ForwardRef("Brand")
Button = ForwardRef("Button")
CallArgument = ForwardRef("CallArgument")
CallBlock = ForwardRef("CallBlock")
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
CompilationMessage = ForwardRef("CompilationMessage")
ConstantValidator = ForwardRef("ConstantValidator")
ContactPoint = ForwardRef("ContactPoint")
Cord = ForwardRef("Cord")
CreativeWork = ForwardRef("CreativeWork")
Datatable = ForwardRef("Datatable")
DatatableColumn = ForwardRef("DatatableColumn")
DatatableColumnHint = ForwardRef("DatatableColumnHint")
DatatableHint = ForwardRef("DatatableHint")
Date = ForwardRef("Date")
DateTime = ForwardRef("DateTime")
DateTimeValidator = ForwardRef("DateTimeValidator")
DateValidator = ForwardRef("DateValidator")
DefinedTerm = ForwardRef("DefinedTerm")
DeleteBlock = ForwardRef("DeleteBlock")
DeleteInline = ForwardRef("DeleteInline")
Directory = ForwardRef("Directory")
Duration = ForwardRef("Duration")
DurationValidator = ForwardRef("DurationValidator")
Emphasis = ForwardRef("Emphasis")
EnumValidator = ForwardRef("EnumValidator")
Enumeration = ForwardRef("Enumeration")
ExecutionDependant = ForwardRef("ExecutionDependant")
ExecutionDependency = ForwardRef("ExecutionDependency")
ExecutionMessage = ForwardRef("ExecutionMessage")
ExecutionTag = ForwardRef("ExecutionTag")
Figure = ForwardRef("Figure")
File = ForwardRef("File")
ForBlock = ForwardRef("ForBlock")
Form = ForwardRef("Form")
Function = ForwardRef("Function")
Grant = ForwardRef("Grant")
Heading = ForwardRef("Heading")
IfBlock = ForwardRef("IfBlock")
IfBlockClause = ForwardRef("IfBlockClause")
ImageObject = ForwardRef("ImageObject")
IncludeBlock = ForwardRef("IncludeBlock")
InsertBlock = ForwardRef("InsertBlock")
InsertInline = ForwardRef("InsertInline")
InstructionBlock = ForwardRef("InstructionBlock")
InstructionInline = ForwardRef("InstructionInline")
InstructionMessage = ForwardRef("InstructionMessage")
IntegerValidator = ForwardRef("IntegerValidator")
Link = ForwardRef("Link")
List = ForwardRef("List")
ListItem = ForwardRef("ListItem")
MathBlock = ForwardRef("MathBlock")
MathInline = ForwardRef("MathInline")
MediaObject = ForwardRef("MediaObject")
ModifyBlock = ForwardRef("ModifyBlock")
ModifyInline = ForwardRef("ModifyInline")
ModifyOperation = ForwardRef("ModifyOperation")
MonetaryGrant = ForwardRef("MonetaryGrant")
Note = ForwardRef("Note")
NumberValidator = ForwardRef("NumberValidator")
Object = ForwardRef("Object")
ObjectHint = ForwardRef("ObjectHint")
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
ReplaceBlock = ForwardRef("ReplaceBlock")
ReplaceInline = ForwardRef("ReplaceInline")
Review = ForwardRef("Review")
Section = ForwardRef("Section")
SoftwareApplication = ForwardRef("SoftwareApplication")
SoftwareSourceCode = ForwardRef("SoftwareSourceCode")
Strikeout = ForwardRef("Strikeout")
StringHint = ForwardRef("StringHint")
StringOperation = ForwardRef("StringOperation")
StringPatch = ForwardRef("StringPatch")
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
Unknown = ForwardRef("Unknown")
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
