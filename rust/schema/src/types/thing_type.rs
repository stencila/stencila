// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::article::Article;
use super::audio_object::AudioObject;
use super::brand::Brand;
use super::chat::Chat;
use super::claim::Claim;
use super::collection::Collection;
use super::comment::Comment;
use super::contact_point::ContactPoint;
use super::creative_work::CreativeWork;
use super::datatable::Datatable;
use super::defined_term::DefinedTerm;
use super::enumeration::Enumeration;
use super::figure::Figure;
use super::grant::Grant;
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
use super::software_application::SoftwareApplication;
use super::software_source_code::SoftwareSourceCode;
use super::table::Table;
use super::video_object::VideoObject;

/// Union type for all types that are descended from `Thing`
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(untagged, crate = "common::serde")]
pub enum ThingType {
    #[default]
    Article(Article),

    AudioObject(AudioObject),

    Brand(Brand),

    Chat(Chat),

    Claim(Claim),

    Collection(Collection),

    Comment(Comment),

    ContactPoint(ContactPoint),

    CreativeWork(CreativeWork),

    Datatable(Datatable),

    DefinedTerm(DefinedTerm),

    Enumeration(Enumeration),

    Figure(Figure),

    Grant(Grant),

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

    SoftwareApplication(SoftwareApplication),

    SoftwareSourceCode(SoftwareSourceCode),

    Table(Table),

    VideoObject(VideoObject),
}
