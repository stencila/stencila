// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::admonition::Admonition;
use super::array::Array;
use super::array_validator::ArrayValidator;
use super::article::Article;
use super::audio_object::AudioObject;
use super::boolean::Boolean;
use super::boolean_validator::BooleanValidator;
use super::brand::Brand;
use super::button::Button;
use super::call::Call;
use super::call_argument::CallArgument;
use super::cite::Cite;
use super::cite_group::CiteGroup;
use super::claim::Claim;
use super::code_block::CodeBlock;
use super::code_chunk::CodeChunk;
use super::code_expression::CodeExpression;
use super::code_fragment::CodeFragment;
use super::code_location::CodeLocation;
use super::collection::Collection;
use super::comment::Comment;
use super::compilation_digest::CompilationDigest;
use super::compilation_error::CompilationError;
use super::constant_validator::ConstantValidator;
use super::contact_point::ContactPoint;
use super::cord::Cord;
use super::creative_work::CreativeWork;
use super::datatable::Datatable;
use super::datatable_column::DatatableColumn;
use super::date::Date;
use super::date_time::DateTime;
use super::date_time_validator::DateTimeValidator;
use super::date_validator::DateValidator;
use super::defined_term::DefinedTerm;
use super::delete::Delete;
use super::directory::Directory;
use super::division::Division;
use super::duration::Duration;
use super::duration_validator::DurationValidator;
use super::emphasis::Emphasis;
use super::enum_validator::EnumValidator;
use super::enumeration::Enumeration;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_error::ExecutionError;
use super::execution_tag::ExecutionTag;
use super::figure::Figure;
use super::file::File;
use super::r#for::For;
use super::form::Form;
use super::function::Function;
use super::grant::Grant;
use super::heading::Heading;
use super::r#if::If;
use super::if_clause::IfClause;
use super::image_object::ImageObject;
use super::include::Include;
use super::insert::Insert;
use super::integer::Integer;
use super::integer_validator::IntegerValidator;
use super::link::Link;
use super::list::List;
use super::list_item::ListItem;
use super::math_block::MathBlock;
use super::math_fragment::MathFragment;
use super::media_object::MediaObject;
use super::monetary_grant::MonetaryGrant;
use super::note::Note;
use super::null::Null;
use super::number::Number;
use super::number_validator::NumberValidator;
use super::object::Object;
use super::organization::Organization;
use super::paragraph::Paragraph;
use super::parameter::Parameter;
use super::periodical::Periodical;
use super::person::Person;
use super::postal_address::PostalAddress;
use super::product::Product;
use super::property_value::PropertyValue;
use super::publication_issue::PublicationIssue;
use super::publication_volume::PublicationVolume;
use super::quote::Quote;
use super::quote_block::QuoteBlock;
use super::review::Review;
use super::section::Section;
use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::span::Span;
use super::strikeout::Strikeout;
use super::string::String;
use super::string_validator::StringValidator;
use super::strong::Strong;
use super::subscript::Subscript;
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
use super::unsigned_integer::UnsignedInteger;
use super::variable::Variable;
use super::video_object::VideoObject;

/// Union type for all types in this schema, including primitives and entities
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault)]
#[serde(untagged, crate = "common::serde")]
pub enum Node {
    #[default]
    Null(Null),

    Boolean(Boolean),

    Integer(Integer),

    UnsignedInteger(UnsignedInteger),

    Number(Number),

    String(String),

    Cord(Cord),

    Array(Array),

    Admonition(Admonition),

    ArrayValidator(ArrayValidator),

    Article(Article),

    AudioObject(AudioObject),

    BooleanValidator(BooleanValidator),

    Brand(Brand),

    Button(Button),

    Call(Call),

    CallArgument(CallArgument),

    Cite(Cite),

    CiteGroup(CiteGroup),

    Claim(Claim),

    CodeBlock(CodeBlock),

    CodeChunk(CodeChunk),

    CodeExpression(CodeExpression),

    CodeFragment(CodeFragment),

    CodeLocation(CodeLocation),

    Collection(Collection),

    Comment(Comment),

    CompilationDigest(CompilationDigest),

    CompilationError(CompilationError),

    ConstantValidator(ConstantValidator),

    ContactPoint(ContactPoint),

    CreativeWork(CreativeWork),

    Datatable(Datatable),

    DatatableColumn(DatatableColumn),

    Date(Date),

    DateTime(DateTime),

    DateTimeValidator(DateTimeValidator),

    DateValidator(DateValidator),

    DefinedTerm(DefinedTerm),

    Delete(Delete),

    Directory(Directory),

    Division(Division),

    Duration(Duration),

    DurationValidator(DurationValidator),

    Emphasis(Emphasis),

    EnumValidator(EnumValidator),

    Enumeration(Enumeration),

    ExecutionDependant(ExecutionDependant),

    ExecutionDependency(ExecutionDependency),

    ExecutionError(ExecutionError),

    ExecutionTag(ExecutionTag),

    Figure(Figure),

    File(File),

    For(For),

    Form(Form),

    Function(Function),

    Grant(Grant),

    Heading(Heading),

    If(If),

    IfClause(IfClause),

    ImageObject(ImageObject),

    Include(Include),

    Insert(Insert),

    IntegerValidator(IntegerValidator),

    Link(Link),

    List(List),

    ListItem(ListItem),

    MathBlock(MathBlock),

    MathFragment(MathFragment),

    MediaObject(MediaObject),

    MonetaryGrant(MonetaryGrant),

    Note(Note),

    NumberValidator(NumberValidator),

    Organization(Organization),

    Paragraph(Paragraph),

    Parameter(Parameter),

    Periodical(Periodical),

    Person(Person),

    PostalAddress(PostalAddress),

    Product(Product),

    PropertyValue(PropertyValue),

    PublicationIssue(PublicationIssue),

    PublicationVolume(PublicationVolume),

    Quote(Quote),

    QuoteBlock(QuoteBlock),

    Review(Review),

    Section(Section),

    SoftwareApplication(SoftwareApplication),

    SoftwareSourceCode(SoftwareSourceCode),

    Span(Span),

    Strikeout(Strikeout),

    StringValidator(StringValidator),

    Strong(Strong),

    Subscript(Subscript),

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

    Variable(Variable),

    VideoObject(VideoObject),

    Object(Object),
}
