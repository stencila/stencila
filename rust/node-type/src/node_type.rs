// Generated file; do not edit. See `schema-gen` crate.

use common::{
    serde::Serialize,
    strum::{Display, EnumIter, EnumString},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Display, EnumString, EnumIter)]
#[serde(crate = "common::serde")]
#[strum(crate = "common::strum")]
pub enum NodeType {
    Null,
    Boolean,
    Integer,
    UnsignedInteger,
    Number,
    String,
    Cord,
    Array,
    Admonition,
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
    ExecutionError,
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
    IntegerValidator,
    Link,
    List,
    ListItem,
    MathBlock,
    MathInline,
    MediaObject,
    Message,
    ModifyBlock,
    ModifyInline,
    ModifyOperation,
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
    ReplaceBlock,
    ReplaceInline,
    Review,
    Section,
    SoftwareApplication,
    SoftwareSourceCode,
    Strikeout,
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
    Variable,
    VideoObject,
    Object,
}
