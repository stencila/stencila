// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::admonition_type::AdmonitionType;
use super::article::Article;
use super::assistant::Assistant;
use super::audio_object::AudioObject;
use super::author_role_name::AuthorRoleName;
use super::brand::Brand;
use super::citation_intent::CitationIntent;
use super::citation_mode::CitationMode;
use super::claim::Claim;
use super::claim_type::ClaimType;
use super::collection::Collection;
use super::comment::Comment;
use super::contact_point::ContactPoint;
use super::creative_work::CreativeWork;
use super::datatable::Datatable;
use super::defined_term::DefinedTerm;
use super::enumeration::Enumeration;
use super::execution_dependant_relation::ExecutionDependantRelation;
use super::execution_dependency_relation::ExecutionDependencyRelation;
use super::execution_mode::ExecutionMode;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::figure::Figure;
use super::form_derive_action::FormDeriveAction;
use super::grant::Grant;
use super::image_object::ImageObject;
use super::instruction_type::InstructionType;
use super::label_type::LabelType;
use super::list_item::ListItem;
use super::list_order::ListOrder;
use super::media_object::MediaObject;
use super::message_level::MessageLevel;
use super::message_role::MessageRole;
use super::monetary_grant::MonetaryGrant;
use super::note_type::NoteType;
use super::organization::Organization;
use super::periodical::Periodical;
use super::person::Person;
use super::postal_address::PostalAddress;
use super::product::Product;
use super::property_value::PropertyValue;
use super::provenance_category::ProvenanceCategory;
use super::publication_issue::PublicationIssue;
use super::publication_volume::PublicationVolume;
use super::review::Review;
use super::section_type::SectionType;
use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::suggestion_status::SuggestionStatus;
use super::table::Table;
use super::table_cell_type::TableCellType;
use super::table_row_type::TableRowType;
use super::time_unit::TimeUnit;
use super::video_object::VideoObject;

/// Union type for all types that are descended from `Thing`
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum ThingType {
    #[default]
    AdmonitionType(AdmonitionType),

    Article(Article),

    Assistant(Assistant),

    AudioObject(AudioObject),

    AuthorRoleName(AuthorRoleName),

    Brand(Brand),

    CitationIntent(CitationIntent),

    CitationMode(CitationMode),

    Claim(Claim),

    ClaimType(ClaimType),

    Collection(Collection),

    Comment(Comment),

    ContactPoint(ContactPoint),

    CreativeWork(CreativeWork),

    Datatable(Datatable),

    DefinedTerm(DefinedTerm),

    Enumeration(Enumeration),

    ExecutionDependantRelation(ExecutionDependantRelation),

    ExecutionDependencyRelation(ExecutionDependencyRelation),

    ExecutionMode(ExecutionMode),

    ExecutionRequired(ExecutionRequired),

    ExecutionStatus(ExecutionStatus),

    Figure(Figure),

    FormDeriveAction(FormDeriveAction),

    Grant(Grant),

    ImageObject(ImageObject),

    InstructionType(InstructionType),

    LabelType(LabelType),

    ListItem(ListItem),

    ListOrder(ListOrder),

    MediaObject(MediaObject),

    MessageLevel(MessageLevel),

    MessageRole(MessageRole),

    MonetaryGrant(MonetaryGrant),

    NoteType(NoteType),

    Organization(Organization),

    Periodical(Periodical),

    Person(Person),

    PostalAddress(PostalAddress),

    Product(Product),

    PropertyValue(PropertyValue),

    ProvenanceCategory(ProvenanceCategory),

    PublicationIssue(PublicationIssue),

    PublicationVolume(PublicationVolume),

    Review(Review),

    SectionType(SectionType),

    SoftwareApplication(SoftwareApplication),

    SoftwareSourceCode(SoftwareSourceCode),

    SuggestionStatus(SuggestionStatus),

    Table(Table),

    TableCellType(TableCellType),

    TableRowType(TableRowType),

    TimeUnit(TimeUnit),

    VideoObject(VideoObject),
}
