// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::action::Action;
use super::agent::Agent;
use super::article::Article;
use super::audio_object::AudioObject;
use super::brand::Brand;
use super::chat::Chat;
use super::claim::Claim;
use super::collection::Collection;
use super::comment::Comment;
use super::contact_point::ContactPoint;
use super::convert_action::ConvertAction;
use super::create_action::CreateAction;
use super::creative_work::CreativeWork;
use super::datatable::Datatable;
use super::defined_term::DefinedTerm;
use super::enumeration::Enumeration;
use super::execute_action::ExecuteAction;
use super::figure::Figure;
use super::file::File;
use super::grant::Grant;
use super::graph::Graph;
use super::image_object::ImageObject;
use super::list_item::ListItem;
use super::media_object::MediaObject;
use super::monetary_grant::MonetaryGrant;
use super::organization::Organization;
use super::periodical::Periodical;
use super::person::Person;
use super::postal_address::PostalAddress;
use super::product::Product;
use super::prompt::Prompt;
use super::property_value::PropertyValue;
use super::publication_issue::PublicationIssue;
use super::publication_volume::PublicationVolume;
use super::review::Review;
use super::skill::Skill;
use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::table::Table;
use super::video_object::VideoObject;
use super::workflow::Workflow;

/// Union type for all types that are descended from `Thing`
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(untagged)]
pub enum ThingVariant {
    #[default]
    Action(Action),

    Agent(Agent),

    Article(Article),

    AudioObject(AudioObject),

    Brand(Brand),

    Chat(Chat),

    Claim(Claim),

    Collection(Collection),

    Comment(Comment),

    ContactPoint(ContactPoint),

    ConvertAction(ConvertAction),

    CreateAction(CreateAction),

    CreativeWork(CreativeWork),

    Datatable(Datatable),

    DefinedTerm(DefinedTerm),

    Enumeration(Enumeration),

    ExecuteAction(ExecuteAction),

    Figure(Figure),

    File(File),

    Grant(Grant),

    Graph(Graph),

    ImageObject(ImageObject),

    ListItem(ListItem),

    MediaObject(MediaObject),

    MonetaryGrant(MonetaryGrant),

    Organization(Organization),

    Periodical(Periodical),

    Person(Person),

    PostalAddress(PostalAddress),

    Product(Product),

    Prompt(Prompt),

    PropertyValue(PropertyValue),

    PublicationIssue(PublicationIssue),

    PublicationVolume(PublicationVolume),

    Review(Review),

    Skill(Skill),

    SoftwareApplication(SoftwareApplication),

    SoftwareSourceCode(SoftwareSourceCode),

    Table(Table),

    VideoObject(VideoObject),

    Workflow(Workflow),
}
