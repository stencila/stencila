use crate::{CreativeWorkType, Inline, Node, prelude::*, replicate};

impl CreativeWorkType {
    // TODO: add similar methods for doi(), authors etc, OR have a
    // CreativeWorkTrait and implement that for each subtype.

    // Get the title of the creative work
    pub fn title(&self) -> Option<Vec<Inline>> {
        macro_rules! works {
            ($( $variant:ident ),*) => {
                match self {
                   CreativeWorkType::Article(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkType::AudioObject(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkType::ImageObject(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkType::VideoObject(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkType::Chat(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkType::Prompt(node) => replicate(&node.title).ok(),
                   $(CreativeWorkType::$variant(node) => node.options.title.as_ref().and_then(|title| replicate(title).ok()),)*
                }
            };
        }
        works!(
            Claim,
            Collection,
            Comment,
            Datatable,
            Figure,
            MediaObject,
            Periodical,
            PublicationIssue,
            PublicationVolume,
            Review,
            SoftwareApplication,
            SoftwareSourceCode,
            Table
        )
    }
}

impl TryFrom<Node> for CreativeWorkType {
    type Error = ErrReport;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        macro_rules! works {
            ($( $variant:ident ),*) => {
                match node {
                    $(Node::$variant(node) => Ok(CreativeWorkType::$variant(node)),)*
                    _ => bail!("Unable to convert Node::{} to CreativeWorkType", node.node_type())
                }
            };
        }
        works!(
            Article,
            AudioObject,
            Chat,
            Claim,
            Collection,
            Comment,
            Datatable,
            Figure,
            ImageObject,
            MediaObject,
            Periodical,
            Prompt,
            PublicationIssue,
            PublicationVolume,
            Review,
            SoftwareApplication,
            SoftwareSourceCode,
            Table,
            VideoObject
        )
    }
}
