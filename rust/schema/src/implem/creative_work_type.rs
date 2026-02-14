use crate::{CreativeWorkType, CreativeWorkVariant, Inline, Node, prelude::*, replicate};

impl CreativeWorkVariant {
    // Get the type of the creative work
    pub fn work_type(&self) -> CreativeWorkType {
        match self {
            CreativeWorkVariant::Agent(..) => CreativeWorkType::Agent,
            CreativeWorkVariant::Article(..) => CreativeWorkType::Article,
            CreativeWorkVariant::AudioObject(..) => CreativeWorkType::AudioObject,
            CreativeWorkVariant::Chat(..) => CreativeWorkType::Chat,
            CreativeWorkVariant::Claim(..) => CreativeWorkType::Claim,
            CreativeWorkVariant::Collection(..) => CreativeWorkType::Collection,
            CreativeWorkVariant::Comment(..) => CreativeWorkType::Comment,
            CreativeWorkVariant::Datatable(..) => CreativeWorkType::Datatable,
            CreativeWorkVariant::Figure(..) => CreativeWorkType::Figure,
            CreativeWorkVariant::File(..) => CreativeWorkType::File,
            CreativeWorkVariant::ImageObject(..) => CreativeWorkType::ImageObject,
            CreativeWorkVariant::MediaObject(..) => CreativeWorkType::MediaObject,
            CreativeWorkVariant::Periodical(..) => CreativeWorkType::Periodical,
            CreativeWorkVariant::Prompt(..) => CreativeWorkType::Prompt,
            CreativeWorkVariant::PublicationIssue(..) => CreativeWorkType::PublicationIssue,
            CreativeWorkVariant::PublicationVolume(..) => CreativeWorkType::PublicationVolume,
            CreativeWorkVariant::Review(..) => CreativeWorkType::Review,
            CreativeWorkVariant::SoftwareApplication(..) => CreativeWorkType::SoftwareApplication,
            CreativeWorkVariant::SoftwareSourceCode(..) => CreativeWorkType::SoftwareSourceCode,
            CreativeWorkVariant::Table(..) => CreativeWorkType::Table,
            CreativeWorkVariant::Skill(..) => CreativeWorkType::Skill,
            CreativeWorkVariant::VideoObject(..) => CreativeWorkType::VideoObject,
        }
    }

    // Get the DOI of the creative work
    pub fn doi(&self) -> Option<String> {
        macro_rules! works {
            ($( $variant:ident ),*) => {
                match self {
                   $(CreativeWorkVariant::$variant(node) => node.doi.clone(),)*
                }
            };
        }
        works!(
            Agent,
            Article,
            AudioObject,
            Chat,
            Claim,
            Collection,
            Comment,
            Datatable,
            Figure,
            File,
            ImageObject,
            MediaObject,
            Periodical,
            Prompt,
            PublicationIssue,
            PublicationVolume,
            Review,
            Skill,
            SoftwareApplication,
            SoftwareSourceCode,
            Table,
            VideoObject
        )
    }

    // Get the title of the creative work
    pub fn title(&self) -> Option<Vec<Inline>> {
        fn replicate_option(title: &Option<Vec<Inline>>) -> Option<Vec<Inline>> {
            title.as_ref().and_then(|title| replicate(title).ok())
        }

        macro_rules! works {
            ($( $variant:ident ),*) => {
                match self {
                   CreativeWorkVariant::Article(node) => replicate_option(&node.title),
                   CreativeWorkVariant::AudioObject(node) => replicate_option(&node.title),
                   CreativeWorkVariant::ImageObject(node) => replicate_option(&node.title),
                   CreativeWorkVariant::VideoObject(node) => replicate_option(&node.title),
                   CreativeWorkVariant::Chat(node) => replicate_option(&node.title),
                   CreativeWorkVariant::Prompt(node) => replicate(&node.title).ok(),
                   $(CreativeWorkVariant::$variant(node) => replicate_option(&node.options.title),)*
                }
            };
        }
        works!(
            Agent,
            Claim,
            Collection,
            Comment,
            Datatable,
            Figure,
            File,
            MediaObject,
            Periodical,
            PublicationIssue,
            PublicationVolume,
            Review,
            Skill,
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
            Agent,
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
            Skill,
            SoftwareApplication,
            SoftwareSourceCode,
            Table,
            VideoObject
        )
    }
}
