// Generated file; do not edit. See `schema-gen` crate.

use common::{
    serde::Serialize,
    strum::{Display, EnumIter, EnumString},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Display, EnumString, EnumIter)]
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
    Chat,
    ChatMessage,
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
    Config,
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
    InstructionModel,
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
    Prompt,
    PromptBlock,
    PropertyValue,
    ProvenanceCount,
    PublicationIssue,
    PublicationVolume,
    QuoteBlock,
    QuoteInline,
    RawBlock,
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
    SuggestionBlock,
    SuggestionInline,
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
    Walkthrough,
    WalkthroughStep,
    Object,
}
