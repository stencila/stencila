// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::admonition::Admonition;
use super::annotation::Annotation;
use super::appendix_break::AppendixBreak;
use super::array::Array;
use super::array_hint::ArrayHint;
use super::array_validator::ArrayValidator;
use super::article::Article;
use super::audio_object::AudioObject;
use super::author_role::AuthorRole;
use super::boolean::Boolean;
use super::boolean_validator::BooleanValidator;
use super::brand::Brand;
use super::button::Button;
use super::call_argument::CallArgument;
use super::call_block::CallBlock;
use super::chat::Chat;
use super::chat_message::ChatMessage;
use super::chat_message_group::ChatMessageGroup;
use super::citation::Citation;
use super::citation_group::CitationGroup;
use super::claim::Claim;
use super::code_block::CodeBlock;
use super::code_chunk::CodeChunk;
use super::code_expression::CodeExpression;
use super::code_inline::CodeInline;
use super::code_location::CodeLocation;
use super::collection::Collection;
use super::comment::Comment;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::constant_validator::ConstantValidator;
use super::contact_point::ContactPoint;
use super::cord::Cord;
use super::creative_work::CreativeWork;
use super::datatable::Datatable;
use super::datatable_column::DatatableColumn;
use super::datatable_column_hint::DatatableColumnHint;
use super::datatable_hint::DatatableHint;
use super::date::Date;
use super::date_time::DateTime;
use super::date_time_validator::DateTimeValidator;
use super::date_validator::DateValidator;
use super::defined_term::DefinedTerm;
use super::directory::Directory;
use super::duration::Duration;
use super::duration_validator::DurationValidator;
use super::emphasis::Emphasis;
use super::enum_validator::EnumValidator;
use super::enumeration::Enumeration;
use super::excerpt::Excerpt;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_message::ExecutionMessage;
use super::execution_tag::ExecutionTag;
use super::figure::Figure;
use super::file::File;
use super::for_block::ForBlock;
use super::form::Form;
use super::function::Function;
use super::grant::Grant;
use super::heading::Heading;
use super::if_block::IfBlock;
use super::if_block_clause::IfBlockClause;
use super::image_object::ImageObject;
use super::include_block::IncludeBlock;
use super::inlines_block::InlinesBlock;
use super::instruction_block::InstructionBlock;
use super::instruction_inline::InstructionInline;
use super::instruction_message::InstructionMessage;
use super::integer::Integer;
use super::integer_validator::IntegerValidator;
use super::island::Island;
use super::link::Link;
use super::list::List;
use super::list_item::ListItem;
use super::math_block::MathBlock;
use super::math_inline::MathInline;
use super::media_object::MediaObject;
use super::model_parameters::ModelParameters;
use super::monetary_grant::MonetaryGrant;
use super::note::Note;
use super::null::Null;
use super::number::Number;
use super::number_validator::NumberValidator;
use super::object::Object;
use super::object_hint::ObjectHint;
use super::organization::Organization;
use super::paragraph::Paragraph;
use super::parameter::Parameter;
use super::periodical::Periodical;
use super::person::Person;
use super::postal_address::PostalAddress;
use super::product::Product;
use super::prompt::Prompt;
use super::prompt_block::PromptBlock;
use super::property_value::PropertyValue;
use super::provenance_count::ProvenanceCount;
use super::publication_issue::PublicationIssue;
use super::publication_volume::PublicationVolume;
use super::quote_block::QuoteBlock;
use super::quote_inline::QuoteInline;
use super::raw_block::RawBlock;
use super::reference::Reference;
use super::review::Review;
use super::section::Section;
use super::sentence::Sentence;
use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::strikeout::Strikeout;
use super::string::String;
use super::string_hint::StringHint;
use super::string_validator::StringValidator;
use super::strong::Strong;
use super::styled_block::StyledBlock;
use super::styled_inline::StyledInline;
use super::subscript::Subscript;
use super::suggestion_block::SuggestionBlock;
use super::suggestion_inline::SuggestionInline;
use super::superscript::Superscript;
use super::table::Table;
use super::table_cell::TableCell;
use super::table_row::TableRow;
use super::text::Text;
use super::thematic_break::ThematicBreak;
use super::thing::Thing;
use super::time::Time;
use super::time_validator::TimeValidator;
use super::timestamp::Timestamp;
use super::timestamp_validator::TimestampValidator;
use super::tuple_validator::TupleValidator;
use super::underline::Underline;
use super::unknown::Unknown;
use super::unsigned_integer::UnsignedInteger;
use super::variable::Variable;
use super::video_object::VideoObject;
use super::walkthrough::Walkthrough;
use super::walkthrough_step::WalkthroughStep;

/// Union type for all types in this schema, including primitives and entities
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
pub enum Node {
    #[default]
    Null(Null),

    Boolean(Boolean),

    Integer(Integer),

    UnsignedInteger(UnsignedInteger),

    Number(Number),

    String(String),

    Array(Array),

    Admonition(Admonition),

    Annotation(Annotation),

    AppendixBreak(AppendixBreak),

    ArrayHint(ArrayHint),

    ArrayValidator(ArrayValidator),

    Article(Article),

    AudioObject(AudioObject),

    AuthorRole(AuthorRole),

    BooleanValidator(BooleanValidator),

    Brand(Brand),

    Button(Button),

    CallArgument(CallArgument),

    CallBlock(CallBlock),

    Chat(Chat),

    ChatMessage(ChatMessage),

    ChatMessageGroup(ChatMessageGroup),

    Citation(Citation),

    CitationGroup(CitationGroup),

    Claim(Claim),

    CodeBlock(CodeBlock),

    CodeChunk(CodeChunk),

    CodeExpression(CodeExpression),

    CodeInline(CodeInline),

    CodeLocation(CodeLocation),

    Collection(Collection),

    Comment(Comment),

    CompilationDigest(CompilationDigest),

    CompilationMessage(CompilationMessage),

    ConstantValidator(ConstantValidator),

    ContactPoint(ContactPoint),

    CreativeWork(CreativeWork),

    Datatable(Datatable),

    DatatableColumn(DatatableColumn),

    DatatableColumnHint(DatatableColumnHint),

    DatatableHint(DatatableHint),

    Date(Date),

    DateTime(DateTime),

    DateTimeValidator(DateTimeValidator),

    DateValidator(DateValidator),

    DefinedTerm(DefinedTerm),

    Directory(Directory),

    Duration(Duration),

    DurationValidator(DurationValidator),

    Emphasis(Emphasis),

    EnumValidator(EnumValidator),

    Enumeration(Enumeration),

    Excerpt(Excerpt),

    ExecutionDependant(ExecutionDependant),

    ExecutionDependency(ExecutionDependency),

    ExecutionMessage(ExecutionMessage),

    ExecutionTag(ExecutionTag),

    Figure(Figure),

    File(File),

    ForBlock(ForBlock),

    Form(Form),

    Function(Function),

    Grant(Grant),

    Heading(Heading),

    IfBlock(IfBlock),

    IfBlockClause(IfBlockClause),

    ImageObject(ImageObject),

    IncludeBlock(IncludeBlock),

    InlinesBlock(InlinesBlock),

    InstructionBlock(InstructionBlock),

    InstructionInline(InstructionInline),

    InstructionMessage(InstructionMessage),

    IntegerValidator(IntegerValidator),

    Island(Island),

    Link(Link),

    List(List),

    ListItem(ListItem),

    MathBlock(MathBlock),

    MathInline(MathInline),

    MediaObject(MediaObject),

    ModelParameters(ModelParameters),

    MonetaryGrant(MonetaryGrant),

    Note(Note),

    NumberValidator(NumberValidator),

    ObjectHint(ObjectHint),

    Organization(Organization),

    Paragraph(Paragraph),

    Parameter(Parameter),

    Periodical(Periodical),

    Person(Person),

    PostalAddress(PostalAddress),

    Product(Product),

    Prompt(Prompt),

    PromptBlock(PromptBlock),

    PropertyValue(PropertyValue),

    ProvenanceCount(ProvenanceCount),

    PublicationIssue(PublicationIssue),

    PublicationVolume(PublicationVolume),

    QuoteBlock(QuoteBlock),

    QuoteInline(QuoteInline),

    RawBlock(RawBlock),

    Reference(Reference),

    Review(Review),

    Section(Section),

    Sentence(Sentence),

    SoftwareApplication(SoftwareApplication),

    SoftwareSourceCode(SoftwareSourceCode),

    Strikeout(Strikeout),

    StringHint(StringHint),

    StringValidator(StringValidator),

    Strong(Strong),

    StyledBlock(StyledBlock),

    StyledInline(StyledInline),

    Subscript(Subscript),

    SuggestionBlock(SuggestionBlock),

    SuggestionInline(SuggestionInline),

    Superscript(Superscript),

    Table(Table),

    TableCell(TableCell),

    TableRow(TableRow),

    Text(Text),

    ThematicBreak(ThematicBreak),

    Thing(Thing),

    Time(Time),

    TimeValidator(TimeValidator),

    Timestamp(Timestamp),

    TimestampValidator(TimestampValidator),

    TupleValidator(TupleValidator),

    Underline(Underline),

    Unknown(Unknown),

    Variable(Variable),

    VideoObject(VideoObject),

    Walkthrough(Walkthrough),

    WalkthroughStep(WalkthroughStep),

    Cord(Cord),

    Object(Object),
}
