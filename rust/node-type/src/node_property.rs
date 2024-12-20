// Generated file; do not edit. See `schema-gen` crate.

use common::{serde::{Serialize, Deserialize}, strum::{EnumString, Display}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[strum(serialize_all = "camelCase", crate = "common::strum")]
pub enum NodeProperty {
    About,
    Abstract,
    ActiveSuggestion,
    Address,
    AddressCountry,
    AddressLocality,
    AddressRegion,
    AdmonitionType,
    Affiliations,
    AlternateNames,
    Amounts,
    Archive,
    Arguments,
    Author,
    Authors,
    AvailableLanguages,
    Bitrate,
    Brands,
    Caption,
    CellType,
    Cells,
    CharacterCount,
    CharacterPercent,
    Chars,
    CitationIntent,
    CitationMode,
    CitationPrefix,
    CitationSuffix,
    Cite,
    ClaimType,
    ClassList,
    Clauses,
    Code,
    CodeLocation,
    CodeRepository,
    CodeSampleType,
    ColumnSpan,
    Columns,
    CommentAspect,
    Comments,
    CompilationDigest,
    CompilationMessages,
    Config,
    ContactPoints,
    Contains,
    Content,
    ContentSize,
    ContentUrl,
    Contributors,
    CostWeight,
    Css,
    DateAccepted,
    DateCreated,
    DateEnd,
    DateModified,
    DatePublished,
    DateReceived,
    DateStart,
    Default,
    Departments,
    DependantNode,
    DependantRelation,
    DependenciesDigest,
    DependenciesFailed,
    DependenciesStale,
    DependencyNode,
    DependencyRelation,
    DeriveAction,
    DeriveFrom,
    DeriveItem,
    DerivedFrom,
    Description,
    Editors,
    Emails,
    EmbedUrl,
    EndColumn,
    EndLine,
    EndPosition,
    ErrorType,
    ExclusiveMaximum,
    ExclusiveMinimum,
    ExecutionCount,
    ExecutionDependants,
    ExecutionDependencies,
    ExecutionDigest,
    ExecutionDuration,
    ExecutionEnded,
    ExecutionInstance,
    ExecutionKind,
    ExecutionMessages,
    ExecutionMode,
    ExecutionPure,
    ExecutionRequired,
    ExecutionStatus,
    ExecutionTags,
    FamilyNames,
    Feedback,
    Files,
    Format,
    FundedBy,
    FundedItems,
    Funders,
    Genre,
    GivenNames,
    Headings,
    Hint,
    HonorificPrefix,
    HonorificSuffix,
    Id,
    IdPattern,
    Identifiers,
    Images,
    InstructionPatterns,
    InstructionType,
    InstructionTypes,
    IsActive,
    IsChecked,
    IsCollapsed,
    IsDisabled,
    IsEphemeral,
    IsFolded,
    IsGlobal,
    IsInvisible,
    IsPartOf,
    Issns,
    IssueNumber,
    Item,
    ItemReviewed,
    ItemType,
    ItemTypes,
    Items,
    ItemsNullable,
    ItemsValidator,
    Iterations,
    JobTitle,
    Keys,
    Keywords,
    Label,
    LabelAutomatically,
    LabelType,
    LastModified,
    LegalName,
    Length,
    Level,
    Licenses,
    Logo,
    Maintainers,
    MathLanguage,
    Mathml,
    MaxItems,
    MaxLength,
    Maximum,
    MediaType,
    MemberOf,
    Members,
    Message,
    MinItems,
    MinLength,
    Minimum,
    MinimumScore,
    Model,
    MultipleOf,
    Name,
    NativeHint,
    NativeType,
    NodeType,
    NodeTypes,
    NoteType,
    Notes,
    Nulls,
    OperatingSystem,
    Operations,
    Order,
    Otherwise,
    Output,
    Outputs,
    PageEnd,
    PageStart,
    Pagination,
    Parameters,
    ParentItem,
    ParentOrganization,
    Parts,
    Path,
    Pattern,
    Position,
    PostOfficeBoxNumber,
    PostalCode,
    ProductId,
    ProgrammingLanguage,
    Prompt,
    PromptProvided,
    PropertyId,
    Provenance,
    ProvenanceCategory,
    Publisher,
    QualityWeight,
    RandomSeed,
    Recursion,
    References,
    Rel,
    Replacement,
    Replicates,
    Returns,
    ReviewAspect,
    Reviews,
    Role,
    RoleName,
    RowSpan,
    RowType,
    Rows,
    RuntimePlatform,
    SectionType,
    Select,
    SemanticDigest,
    SoftwareRequirements,
    SoftwareVersion,
    Source,
    SpeedWeight,
    Sponsors,
    StackTrace,
    StartColumn,
    StartLine,
    StartPosition,
    StateDigest,
    Steps,
    StreetAddress,
    StyleLanguage,
    SuggestionStatus,
    Suggestions,
    Target,
    TargetProducts,
    TelephoneNumbers,
    Temperature,
    TermCode,
    Text,
    Theme,
    Thumbnail,
    TimeUnit,
    TimeUnits,
    Title,
    Transcript,
    Type,
    UniqueItems,
    Url,
    Validator,
    Value,
    Values,
    Variable,
    Version,
    VolumeNumber,
}
