// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::admonition_type::AdmonitionType;
use super::article::Article;
use super::audio_object::AudioObject;
use super::author_role_name::AuthorRoleName;
use super::automatic_execution::AutomaticExecution;
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
use super::datatable_column::DatatableColumn;
use super::defined_term::DefinedTerm;
use super::directory::Directory;
use super::enumeration::Enumeration;
use super::execution_dependant_relation::ExecutionDependantRelation;
use super::execution_dependency_relation::ExecutionDependencyRelation;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::figure::Figure;
use super::file::File;
use super::form_derive_action::FormDeriveAction;
use super::grant::Grant;
use super::image_object::ImageObject;
use super::list_item::ListItem;
use super::list_order::ListOrder;
use super::media_object::MediaObject;
use super::monetary_grant::MonetaryGrant;
use super::note_type::NoteType;
use super::organization::Organization;
use super::periodical::Periodical;
use super::person::Person;
use super::postal_address::PostalAddress;
use super::product::Product;
use super::property_value::PropertyValue;
use super::publication_issue::PublicationIssue;
use super::publication_volume::PublicationVolume;
use super::review::Review;
use super::section_type::SectionType;
use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::table::Table;
use super::table_cell_type::TableCellType;
use super::table_row_type::TableRowType;
use super::time_unit::TimeUnit;
use super::video_object::VideoObject;

/// Union type for all types that are descended from `Thing`
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(untagged, crate = "common::serde")]
pub enum ThingType {
    #[default]
    AdmonitionType(AdmonitionType),

    Article(Article),

    AudioObject(AudioObject),

    AuthorRoleName(AuthorRoleName),

    AutomaticExecution(AutomaticExecution),

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

    DatatableColumn(DatatableColumn),

    DefinedTerm(DefinedTerm),

    Directory(Directory),

    Enumeration(Enumeration),

    ExecutionDependantRelation(ExecutionDependantRelation),

    ExecutionDependencyRelation(ExecutionDependencyRelation),

    ExecutionRequired(ExecutionRequired),

    ExecutionStatus(ExecutionStatus),

    Figure(Figure),

    File(File),

    FormDeriveAction(FormDeriveAction),

    Grant(Grant),

    ImageObject(ImageObject),

    ListItem(ListItem),

    ListOrder(ListOrder),

    MediaObject(MediaObject),

    MonetaryGrant(MonetaryGrant),

    NoteType(NoteType),

    Organization(Organization),

    Periodical(Periodical),

    Person(Person),

    PostalAddress(PostalAddress),

    Product(Product),

    PropertyValue(PropertyValue),

    PublicationIssue(PublicationIssue),

    PublicationVolume(PublicationVolume),

    Review(Review),

    SectionType(SectionType),

    SoftwareApplication(SoftwareApplication),

    SoftwareSourceCode(SoftwareSourceCode),

    Table(Table),

    TableCellType(TableCellType),

    TableRowType(TableRowType),

    TimeUnit(TimeUnit),

    VideoObject(VideoObject),
}
