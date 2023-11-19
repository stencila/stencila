// Generated file; do not edit. See `schema-gen` crate.
use common::strum::{Display, EnumString, EnumIter};

mod admonition;
mod admonition_type;
mod array;
mod array_validator;
mod article;
mod audio_object;
mod automatic_execution;
mod block;
mod boolean;
mod boolean_validator;
mod brand;
mod button;
mod call_argument;
mod call_block;
mod citation_intent;
mod citation_mode;
mod cite;
mod cite_group;
mod cite_or_text;
mod claim;
mod claim_type;
mod code_block;
mod code_chunk;
mod code_expression;
mod code_inline;
mod code_location;
mod collection;
mod comment;
mod compilation_digest;
mod compilation_error;
mod constant_validator;
mod contact_point;
mod cord;
mod creative_work;
mod creative_work_type;
mod creative_work_type_or_text;
mod datatable;
mod datatable_column;
mod date;
mod date_time;
mod date_time_validator;
mod date_validator;
mod defined_term;
mod delete;
mod directory;
mod duration;
mod duration_validator;
mod emphasis;
mod enum_validator;
mod enumeration;
mod execution_dependant;
mod execution_dependant_node;
mod execution_dependant_relation;
mod execution_dependency;
mod execution_dependency_node;
mod execution_dependency_relation;
mod execution_error;
mod execution_required;
mod execution_status;
mod execution_tag;
mod figure;
mod file;
mod file_or_directory;
mod for_block;
mod form;
mod form_derive_action;
mod function;
mod grant;
mod grant_or_monetary_grant;
mod heading;
mod if_block;
mod if_block_clause;
mod image_object;
mod include_block;
mod inline;
mod insert;
mod integer;
mod integer_or_string;
mod integer_validator;
mod link;
mod list;
mod list_item;
mod list_order;
mod math_block;
mod math_inline;
mod media_object;
mod monetary_grant;
mod node;
mod note;
mod note_type;
mod null;
mod number;
mod number_validator;
mod object;
mod organization;
mod paragraph;
mod parameter;
mod periodical;
mod person;
mod person_or_organization;
mod person_or_organization_or_software_application;
mod postal_address;
mod postal_address_or_string;
mod primitive;
mod product;
mod property_value;
mod property_value_or_string;
mod publication_issue;
mod publication_volume;
mod quote_block;
mod quote_inline;
mod review;
mod section;
mod section_type;
mod software_application;
mod software_source_code;
mod software_source_code_or_software_application_or_string;
mod strikeout;
mod string;
mod string_or_number;
mod string_validator;
mod strong;
mod styled_block;
mod styled_inline;
mod subscript;
mod superscript;
mod table;
mod table_cell;
mod table_cell_type;
mod table_row;
mod table_row_type;
mod text;
mod thematic_break;
mod thing;
mod thing_type;
mod time;
mod time_unit;
mod time_validator;
mod timestamp;
mod timestamp_validator;
mod tuple_validator;
mod underline;
mod unsigned_integer;
mod validator;
mod variable;
mod video_object;

pub use admonition::*;
pub use admonition_type::*;
pub use array::*;
pub use array_validator::*;
pub use article::*;
pub use audio_object::*;
pub use automatic_execution::*;
pub use block::*;
pub use boolean::*;
pub use boolean_validator::*;
pub use brand::*;
pub use button::*;
pub use call_argument::*;
pub use call_block::*;
pub use citation_intent::*;
pub use citation_mode::*;
pub use cite::*;
pub use cite_group::*;
pub use cite_or_text::*;
pub use claim::*;
pub use claim_type::*;
pub use code_block::*;
pub use code_chunk::*;
pub use code_expression::*;
pub use code_inline::*;
pub use code_location::*;
pub use collection::*;
pub use comment::*;
pub use compilation_digest::*;
pub use compilation_error::*;
pub use constant_validator::*;
pub use contact_point::*;
pub use cord::*;
pub use creative_work::*;
pub use creative_work_type::*;
pub use creative_work_type_or_text::*;
pub use datatable::*;
pub use datatable_column::*;
pub use date::*;
pub use date_time::*;
pub use date_time_validator::*;
pub use date_validator::*;
pub use defined_term::*;
pub use delete::*;
pub use directory::*;
pub use duration::*;
pub use duration_validator::*;
pub use emphasis::*;
pub use enum_validator::*;
pub use enumeration::*;
pub use execution_dependant::*;
pub use execution_dependant_node::*;
pub use execution_dependant_relation::*;
pub use execution_dependency::*;
pub use execution_dependency_node::*;
pub use execution_dependency_relation::*;
pub use execution_error::*;
pub use execution_required::*;
pub use execution_status::*;
pub use execution_tag::*;
pub use figure::*;
pub use file::*;
pub use file_or_directory::*;
pub use for_block::*;
pub use form::*;
pub use form_derive_action::*;
pub use function::*;
pub use grant::*;
pub use grant_or_monetary_grant::*;
pub use heading::*;
pub use if_block::*;
pub use if_block_clause::*;
pub use image_object::*;
pub use include_block::*;
pub use inline::*;
pub use insert::*;
pub use integer::*;
pub use integer_or_string::*;
pub use integer_validator::*;
pub use link::*;
pub use list::*;
pub use list_item::*;
pub use list_order::*;
pub use math_block::*;
pub use math_inline::*;
pub use media_object::*;
pub use monetary_grant::*;
pub use node::*;
pub use note::*;
pub use note_type::*;
pub use null::*;
pub use number::*;
pub use number_validator::*;
pub use object::*;
pub use organization::*;
pub use paragraph::*;
pub use parameter::*;
pub use periodical::*;
pub use person::*;
pub use person_or_organization::*;
pub use person_or_organization_or_software_application::*;
pub use postal_address::*;
pub use postal_address_or_string::*;
pub use primitive::*;
pub use product::*;
pub use property_value::*;
pub use property_value_or_string::*;
pub use publication_issue::*;
pub use publication_volume::*;
pub use quote_block::*;
pub use quote_inline::*;
pub use review::*;
pub use section::*;
pub use section_type::*;
pub use software_application::*;
pub use software_source_code::*;
pub use software_source_code_or_software_application_or_string::*;
pub use strikeout::*;
pub use string::*;
pub use string_or_number::*;
pub use string_validator::*;
pub use strong::*;
pub use styled_block::*;
pub use styled_inline::*;
pub use subscript::*;
pub use superscript::*;
pub use table::*;
pub use table_cell::*;
pub use table_cell_type::*;
pub use table_row::*;
pub use table_row_type::*;
pub use text::*;
pub use thematic_break::*;
pub use thing::*;
pub use thing_type::*;
pub use time::*;
pub use time_unit::*;
pub use time_validator::*;
pub use timestamp::*;
pub use timestamp_validator::*;
pub use tuple_validator::*;
pub use underline::*;
pub use unsigned_integer::*;
pub use validator::*;
pub use variable::*;
pub use video_object::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter)]
#[strum(crate="common::strum")]
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
    ForBlock,
    Form,
    Function,
    Grant,
    Heading,
    IfBlock,
    IfBlockClause,
    ImageObject,
    IncludeBlock,
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
    Object
}
