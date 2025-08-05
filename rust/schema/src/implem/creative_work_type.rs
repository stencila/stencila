use crate::{CreativeWorkVariant, Inline, Node, prelude::*, replicate};

impl CreativeWorkVariant {
    // TODO: add similar methods for doi(), authors etc, OR have a
    // CreativeWorkTrait and implement that for each subtype.

    // Get the title of the creative work
    pub fn title(&self) -> Option<Vec<Inline>> {
        macro_rules! works {
            ($( $variant:ident ),*) => {
                match self {
                   CreativeWorkVariant::Article(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkVariant::AudioObject(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkVariant::ImageObject(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkVariant::VideoObject(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkVariant::Chat(node) => node.title.as_ref().and_then(|title| replicate(title).ok()),
                   CreativeWorkVariant::Prompt(node) => replicate(&node.title).ok(),
                   $(CreativeWorkVariant::$variant(node) => node.options.title.as_ref().and_then(|title| replicate(title).ok()),)*
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

impl TryFrom<Node> for CreativeWorkVariant {
    type Error = ErrReport;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        macro_rules! works {
            ($( $variant:ident ),*) => {
                match node {
                    $(Node::$variant(node) => Ok(CreativeWorkVariant::$variant(node)),)*
                    _ => bail!("Unable to convert Node::{} to CreativeWorkVariant", node.node_type())
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
