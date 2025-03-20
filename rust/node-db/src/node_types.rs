// Generated file, do not edit. See the Rust `schema-gen` crate.

use kuzu::{LogicalType, Value};

use codec_text_trait::to_text;
use schema::*;

use super::{DatabaseNode, ToKuzu};

fn relations<'lt, I, D>(iter: I) -> Vec<(NodeType, NodeId)>
where
    I: Iterator<Item = &'lt D>,
    D: DatabaseNode + 'lt,
{
    iter.flat_map(|item| (!matches!(item.node_type(), NodeType::Unknown)).then_some((item.node_type(), item.node_id())))
        .collect()
}

impl DatabaseNode for Admonition {
    fn node_type(&self) -> NodeType {
        NodeType::Admonition
    }

    fn node_id(&self) -> NodeId {
        Admonition::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AdmonitionType, String::to_kuzu_type(), self.admonition_type.to_kuzu_value()),
            (NodeProperty::IsFolded, bool::to_kuzu_type(), self.is_folded.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Title, relations(self.title.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Annotation {
    fn node_type(&self) -> NodeType {
        NodeType::Annotation
    }

    fn node_id(&self) -> NodeId {
        Annotation::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::Annotation, relations(self.annotation.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Article {
    fn node_type(&self) -> NodeType {
        NodeType::Article
    }

    fn node_id(&self) -> NodeId {
        Article::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.keywords.to_kuzu_value()),
            (NodeProperty::ExecutionMode, String::to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, i64::to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, String::to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, String::to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Pagination, String::to_kuzu_type(), self.options.pagination.to_kuzu_value()),
            (NodeProperty::Frontmatter, String::to_kuzu_type(), self.frontmatter.to_kuzu_value()),
            (NodeProperty::Abstract, String::to_kuzu_type(), to_text(&self.r#abstract).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.title.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for AudioObject {
    fn node_type(&self) -> NodeType {
        NodeType::AudioObject
    }

    fn node_id(&self) -> NodeId {
        AudioObject::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Bitrate, f64::to_kuzu_type(), self.options.bitrate.to_kuzu_value()),
            (NodeProperty::ContentSize, f64::to_kuzu_type(), self.options.content_size.to_kuzu_value()),
            (NodeProperty::ContentUrl, String::to_kuzu_type(), self.content_url.to_kuzu_value()),
            (NodeProperty::EmbedUrl, String::to_kuzu_type(), self.options.embed_url.to_kuzu_value()),
            (NodeProperty::MediaType, String::to_kuzu_type(), self.media_type.to_kuzu_value()),
            (NodeProperty::Transcript, String::to_kuzu_type(), self.options.transcript.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.title.iter().flatten())),
            (NodeProperty::Caption, relations(self.caption.iter().flatten()))
        ]
    }
}

impl DatabaseNode for AuthorRole {
    fn node_type(&self) -> NodeType {
        NodeType::AuthorRole
    }

    fn node_id(&self) -> NodeId {
        AuthorRole::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::RoleName, String::to_kuzu_type(), self.role_name.to_kuzu_value()),
            (NodeProperty::Format, String::to_kuzu_type(), self.format.to_kuzu_value()),
            (NodeProperty::LastModified, Timestamp::to_kuzu_type(), self.last_modified.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Author, vec![(self.author.node_type(), self.author.node_id())])
        ]
    }
}

impl DatabaseNode for Brand {
    fn node_type(&self) -> NodeType {
        NodeType::Brand
    }

    fn node_id(&self) -> NodeId {
        Brand::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Reviews, Vec::<String>::to_kuzu_type(), self.options.reviews.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Logo, relations(self.options.logo.iter()))
        ]
    }
}

impl DatabaseNode for Cite {
    fn node_type(&self) -> NodeType {
        NodeType::Cite
    }

    fn node_id(&self) -> NodeId {
        Cite::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Target, String::to_kuzu_type(), self.target.to_kuzu_value()),
            (NodeProperty::CitationMode, String::to_kuzu_type(), self.citation_mode.to_kuzu_value()),
            (NodeProperty::CitationIntent, Vec::<String>::to_kuzu_type(), self.options.citation_intent.to_kuzu_value()),
            (NodeProperty::Pagination, String::to_kuzu_type(), self.options.pagination.to_kuzu_value()),
            (NodeProperty::CitationPrefix, String::to_kuzu_type(), self.options.citation_prefix.to_kuzu_value()),
            (NodeProperty::CitationSuffix, String::to_kuzu_type(), self.options.citation_suffix.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.options.content.iter().flatten()))
        ]
    }
}

impl DatabaseNode for CiteGroup {
    fn node_type(&self) -> NodeType {
        NodeType::CiteGroup
    }

    fn node_id(&self) -> NodeId {
        CiteGroup::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Items, relations(self.items.iter()))
        ]
    }
}

impl DatabaseNode for Claim {
    fn node_type(&self) -> NodeType {
        NodeType::Claim
    }

    fn node_id(&self) -> NodeId {
        Claim::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::ClaimType, String::to_kuzu_type(), self.claim_type.to_kuzu_value()),
            (NodeProperty::Label, String::to_kuzu_type(), self.label.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for CodeBlock {
    fn node_type(&self) -> NodeType {
        NodeType::CodeBlock
    }

    fn node_id(&self) -> NodeId {
        CodeBlock::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for CodeChunk {
    fn node_type(&self) -> NodeType {
        NodeType::CodeChunk
    }

    fn node_id(&self) -> NodeId {
        CodeChunk::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, String::to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, i64::to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, String::to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, String::to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, String::to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, String::to_kuzu_type(), self.options.execution_bounded.to_kuzu_value()),
            (NodeProperty::LabelType, String::to_kuzu_type(), self.label_type.to_kuzu_value()),
            (NodeProperty::Label, String::to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, bool::to_kuzu_type(), self.label_automatically.to_kuzu_value()),
            (NodeProperty::IsEchoed, bool::to_kuzu_type(), self.is_echoed.to_kuzu_value()),
            (NodeProperty::IsHidden, bool::to_kuzu_type(), self.is_hidden.to_kuzu_value()),
            (NodeProperty::ExecutionPure, bool::to_kuzu_type(), self.options.execution_pure.to_kuzu_value()),
            (NodeProperty::Caption, String::to_kuzu_type(), to_text(&self.caption).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Caption, relations(self.caption.iter().flatten()))
        ]
    }
}

impl DatabaseNode for CodeExpression {
    fn node_type(&self) -> NodeType {
        NodeType::CodeExpression
    }

    fn node_id(&self) -> NodeId {
        CodeExpression::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, String::to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, i64::to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, String::to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, String::to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, String::to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, String::to_kuzu_type(), self.options.execution_bounded.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for CodeInline {
    fn node_type(&self) -> NodeType {
        NodeType::CodeInline
    }

    fn node_id(&self) -> NodeId {
        CodeInline::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Collection {
    fn node_type(&self) -> NodeType {
        NodeType::Collection
    }

    fn node_id(&self) -> NodeId {
        Collection::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Comment {
    fn node_type(&self) -> NodeType {
        NodeType::Comment
    }

    fn node_id(&self) -> NodeId {
        Comment::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::CommentAspect, String::to_kuzu_type(), self.options.comment_aspect.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::ParentItem, relations(self.options.parent_item.iter()))
        ]
    }
}

impl DatabaseNode for ContactPoint {
    fn node_type(&self) -> NodeType {
        NodeType::ContactPoint
    }

    fn node_id(&self) -> NodeId {
        ContactPoint::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Emails, Vec::<String>::to_kuzu_type(), self.emails.to_kuzu_value()),
            (NodeProperty::TelephoneNumbers, Vec::<String>::to_kuzu_type(), self.telephone_numbers.to_kuzu_value()),
            (NodeProperty::AvailableLanguages, Vec::<String>::to_kuzu_type(), self.options.available_languages.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten()))
        ]
    }
}

impl DatabaseNode for CreativeWork {
    fn node_type(&self) -> NodeType {
        NodeType::CreativeWork
    }

    fn node_id(&self) -> NodeId {
        CreativeWork::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten()))
        ]
    }
}

impl DatabaseNode for DefinedTerm {
    fn node_type(&self) -> NodeType {
        NodeType::DefinedTerm
    }

    fn node_id(&self) -> NodeId {
        DefinedTerm::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::TermCode, String::to_kuzu_type(), self.options.term_code.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Directory {
    fn node_type(&self) -> NodeType {
        NodeType::Directory
    }

    fn node_id(&self) -> NodeId {
        Directory::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Path, String::to_kuzu_type(), self.path.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            
        ]
    }
}

impl DatabaseNode for Emphasis {
    fn node_type(&self) -> NodeType {
        NodeType::Emphasis
    }

    fn node_id(&self) -> NodeId {
        Emphasis::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for Figure {
    fn node_type(&self) -> NodeType {
        NodeType::Figure
    }

    fn node_id(&self) -> NodeId {
        Figure::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Label, String::to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, bool::to_kuzu_type(), self.label_automatically.to_kuzu_value()),
            (NodeProperty::Caption, String::to_kuzu_type(), to_text(&self.caption).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten())),
            (NodeProperty::Caption, relations(self.caption.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for File {
    fn node_type(&self) -> NodeType {
        NodeType::File
    }

    fn node_id(&self) -> NodeId {
        File::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Path, String::to_kuzu_type(), self.path.to_kuzu_value()),
            (NodeProperty::MediaType, String::to_kuzu_type(), self.media_type.to_kuzu_value()),
            (NodeProperty::TransferEncoding, String::to_kuzu_type(), self.options.transfer_encoding.to_kuzu_value()),
            (NodeProperty::Size, u64::to_kuzu_type(), self.size.to_kuzu_value()),
            (NodeProperty::Content, String::to_kuzu_type(), self.content.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            
        ]
    }
}

impl DatabaseNode for ForBlock {
    fn node_type(&self) -> NodeType {
        NodeType::ForBlock
    }

    fn node_id(&self) -> NodeId {
        ForBlock::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, String::to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, i64::to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, String::to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, String::to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, String::to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, String::to_kuzu_type(), self.options.execution_bounded.to_kuzu_value()),
            (NodeProperty::Variable, String::to_kuzu_type(), self.variable.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::Otherwise, relations(self.otherwise.iter().flatten())),
            (NodeProperty::Iterations, relations(self.iterations.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Function {
    fn node_type(&self) -> NodeType {
        NodeType::Function
    }

    fn node_id(&self) -> NodeId {
        Function::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Parameters, relations(self.parameters.iter()))
        ]
    }
}

impl DatabaseNode for Grant {
    fn node_type(&self) -> NodeType {
        NodeType::Grant
    }

    fn node_id(&self) -> NodeId {
        Grant::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Heading {
    fn node_type(&self) -> NodeType {
        NodeType::Heading
    }

    fn node_id(&self) -> NodeId {
        Heading::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Level, i64::to_kuzu_type(), self.level.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for IfBlock {
    fn node_type(&self) -> NodeType {
        NodeType::IfBlock
    }

    fn node_id(&self) -> NodeId {
        IfBlock::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, String::to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, i64::to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, String::to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, String::to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Clauses, relations(self.clauses.iter()))
        ]
    }
}

impl DatabaseNode for IfBlockClause {
    fn node_type(&self) -> NodeType {
        NodeType::IfBlockClause
    }

    fn node_id(&self) -> NodeId {
        IfBlockClause::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, String::to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, i64::to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, String::to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, String::to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, String::to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, String::to_kuzu_type(), self.options.execution_bounded.to_kuzu_value()),
            (NodeProperty::IsActive, bool::to_kuzu_type(), self.is_active.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for ImageObject {
    fn node_type(&self) -> NodeType {
        NodeType::ImageObject
    }

    fn node_id(&self) -> NodeId {
        ImageObject::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Bitrate, f64::to_kuzu_type(), self.options.bitrate.to_kuzu_value()),
            (NodeProperty::ContentSize, f64::to_kuzu_type(), self.options.content_size.to_kuzu_value()),
            (NodeProperty::ContentUrl, String::to_kuzu_type(), self.content_url.to_kuzu_value()),
            (NodeProperty::EmbedUrl, String::to_kuzu_type(), self.options.embed_url.to_kuzu_value()),
            (NodeProperty::MediaType, String::to_kuzu_type(), self.media_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.title.iter().flatten())),
            (NodeProperty::Caption, relations(self.caption.iter().flatten())),
            (NodeProperty::Thumbnail, relations(self.options.thumbnail.iter()))
        ]
    }
}

impl DatabaseNode for IncludeBlock {
    fn node_type(&self) -> NodeType {
        NodeType::IncludeBlock
    }

    fn node_id(&self) -> NodeId {
        IncludeBlock::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, String::to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, i64::to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, String::to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, String::to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Source, String::to_kuzu_type(), self.source.to_kuzu_value()),
            (NodeProperty::MediaType, String::to_kuzu_type(), self.media_type.to_kuzu_value()),
            (NodeProperty::Select, String::to_kuzu_type(), self.select.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Link {
    fn node_type(&self) -> NodeType {
        NodeType::Link
    }

    fn node_id(&self) -> NodeId {
        Link::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Target, String::to_kuzu_type(), self.target.to_kuzu_value()),
            (NodeProperty::Title, String::to_kuzu_type(), self.title.to_kuzu_value()),
            (NodeProperty::Rel, String::to_kuzu_type(), self.rel.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for List {
    fn node_type(&self) -> NodeType {
        NodeType::List
    }

    fn node_id(&self) -> NodeId {
        List::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Order, String::to_kuzu_type(), self.order.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Items, relations(self.items.iter())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for ListItem {
    fn node_type(&self) -> NodeType {
        NodeType::ListItem
    }

    fn node_id(&self) -> NodeId {
        ListItem::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::IsChecked, bool::to_kuzu_type(), self.is_checked.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for MathBlock {
    fn node_type(&self) -> NodeType {
        NodeType::MathBlock
    }

    fn node_id(&self) -> NodeId {
        MathBlock::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::MathLanguage, String::to_kuzu_type(), self.math_language.to_kuzu_value()),
            (NodeProperty::Label, String::to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, bool::to_kuzu_type(), self.label_automatically.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for MathInline {
    fn node_type(&self) -> NodeType {
        NodeType::MathInline
    }

    fn node_id(&self) -> NodeId {
        MathInline::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::MathLanguage, String::to_kuzu_type(), self.math_language.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for MediaObject {
    fn node_type(&self) -> NodeType {
        NodeType::MediaObject
    }

    fn node_id(&self) -> NodeId {
        MediaObject::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Bitrate, f64::to_kuzu_type(), self.options.bitrate.to_kuzu_value()),
            (NodeProperty::ContentSize, f64::to_kuzu_type(), self.options.content_size.to_kuzu_value()),
            (NodeProperty::ContentUrl, String::to_kuzu_type(), self.content_url.to_kuzu_value()),
            (NodeProperty::EmbedUrl, String::to_kuzu_type(), self.options.embed_url.to_kuzu_value()),
            (NodeProperty::MediaType, String::to_kuzu_type(), self.media_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten()))
        ]
    }
}

impl DatabaseNode for MonetaryGrant {
    fn node_type(&self) -> NodeType {
        NodeType::MonetaryGrant
    }

    fn node_id(&self) -> NodeId {
        MonetaryGrant::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Amounts, f64::to_kuzu_type(), self.options.amounts.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Note {
    fn node_type(&self) -> NodeType {
        NodeType::Note
    }

    fn node_id(&self) -> NodeId {
        Note::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::NoteType, String::to_kuzu_type(), self.note_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for Organization {
    fn node_type(&self) -> NodeType {
        NodeType::Organization
    }

    fn node_id(&self) -> NodeId {
        Organization::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::LegalName, String::to_kuzu_type(), self.options.legal_name.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Brands, relations(self.options.brands.iter().flatten())),
            (NodeProperty::ContactPoints, relations(self.options.contact_points.iter().flatten())),
            (NodeProperty::Departments, relations(self.options.departments.iter().flatten())),
            (NodeProperty::Logo, relations(self.options.logo.iter())),
            (NodeProperty::ParentOrganization, relations(self.options.parent_organization.iter()))
        ]
    }
}

impl DatabaseNode for Paragraph {
    fn node_type(&self) -> NodeType {
        NodeType::Paragraph
    }

    fn node_id(&self) -> NodeId {
        Paragraph::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Text, String::to_kuzu_type(), to_text(self).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Parameter {
    fn node_type(&self) -> NodeType {
        NodeType::Parameter
    }

    fn node_id(&self) -> NodeId {
        Parameter::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, String::to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, i64::to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, String::to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, String::to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Label, String::to_kuzu_type(), self.options.label.to_kuzu_value()),
            (NodeProperty::DerivedFrom, String::to_kuzu_type(), self.options.derived_from.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            
        ]
    }
}

impl DatabaseNode for Periodical {
    fn node_type(&self) -> NodeType {
        NodeType::Periodical
    }

    fn node_id(&self) -> NodeId {
        Periodical::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::DateStart, Date::to_kuzu_type(), self.options.date_start.to_kuzu_value()),
            (NodeProperty::DateEnd, Date::to_kuzu_type(), self.options.date_end.to_kuzu_value()),
            (NodeProperty::Issns, Vec::<String>::to_kuzu_type(), self.options.issns.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Person {
    fn node_type(&self) -> NodeType {
        NodeType::Person
    }

    fn node_id(&self) -> NodeId {
        Person::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Emails, Vec::<String>::to_kuzu_type(), self.options.emails.to_kuzu_value()),
            (NodeProperty::FamilyNames, Vec::<String>::to_kuzu_type(), self.family_names.to_kuzu_value()),
            (NodeProperty::GivenNames, Vec::<String>::to_kuzu_type(), self.given_names.to_kuzu_value()),
            (NodeProperty::HonorificPrefix, String::to_kuzu_type(), self.options.honorific_prefix.to_kuzu_value()),
            (NodeProperty::HonorificSuffix, String::to_kuzu_type(), self.options.honorific_suffix.to_kuzu_value()),
            (NodeProperty::JobTitle, String::to_kuzu_type(), self.options.job_title.to_kuzu_value()),
            (NodeProperty::TelephoneNumbers, Vec::<String>::to_kuzu_type(), self.options.telephone_numbers.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Affiliations, relations(self.affiliations.iter().flatten())),
            (NodeProperty::MemberOf, relations(self.options.member_of.iter().flatten()))
        ]
    }
}

impl DatabaseNode for PostalAddress {
    fn node_type(&self) -> NodeType {
        NodeType::PostalAddress
    }

    fn node_id(&self) -> NodeId {
        PostalAddress::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Emails, Vec::<String>::to_kuzu_type(), self.emails.to_kuzu_value()),
            (NodeProperty::TelephoneNumbers, Vec::<String>::to_kuzu_type(), self.telephone_numbers.to_kuzu_value()),
            (NodeProperty::AvailableLanguages, Vec::<String>::to_kuzu_type(), self.options.available_languages.to_kuzu_value()),
            (NodeProperty::StreetAddress, String::to_kuzu_type(), self.street_address.to_kuzu_value()),
            (NodeProperty::PostOfficeBoxNumber, String::to_kuzu_type(), self.options.post_office_box_number.to_kuzu_value()),
            (NodeProperty::AddressLocality, String::to_kuzu_type(), self.address_locality.to_kuzu_value()),
            (NodeProperty::AddressRegion, String::to_kuzu_type(), self.address_region.to_kuzu_value()),
            (NodeProperty::PostalCode, String::to_kuzu_type(), self.postal_code.to_kuzu_value()),
            (NodeProperty::AddressCountry, String::to_kuzu_type(), self.address_country.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Product {
    fn node_type(&self) -> NodeType {
        NodeType::Product
    }

    fn node_id(&self) -> NodeId {
        Product::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::ProductId, String::to_kuzu_type(), self.options.product_id.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Brands, relations(self.options.brands.iter().flatten())),
            (NodeProperty::Logo, relations(self.options.logo.iter()))
        ]
    }
}

impl DatabaseNode for PropertyValue {
    fn node_type(&self) -> NodeType {
        NodeType::PropertyValue
    }

    fn node_id(&self) -> NodeId {
        PropertyValue::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::PropertyId, String::to_kuzu_type(), self.property_id.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten()))
        ]
    }
}

impl DatabaseNode for PublicationIssue {
    fn node_type(&self) -> NodeType {
        NodeType::PublicationIssue
    }

    fn node_id(&self) -> NodeId {
        PublicationIssue::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Pagination, String::to_kuzu_type(), self.options.pagination.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten()))
        ]
    }
}

impl DatabaseNode for PublicationVolume {
    fn node_type(&self) -> NodeType {
        NodeType::PublicationVolume
    }

    fn node_id(&self) -> NodeId {
        PublicationVolume::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Pagination, String::to_kuzu_type(), self.options.pagination.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten()))
        ]
    }
}

impl DatabaseNode for QuoteBlock {
    fn node_type(&self) -> NodeType {
        NodeType::QuoteBlock
    }

    fn node_id(&self) -> NodeId {
        QuoteBlock::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for QuoteInline {
    fn node_type(&self) -> NodeType {
        NodeType::QuoteInline
    }

    fn node_id(&self) -> NodeId {
        QuoteInline::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for RawBlock {
    fn node_type(&self) -> NodeType {
        NodeType::RawBlock
    }

    fn node_id(&self) -> NodeId {
        RawBlock::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Format, String::to_kuzu_type(), self.format.to_kuzu_value()),
            (NodeProperty::Content, String::to_kuzu_type(), self.content.to_kuzu_value()),
            (NodeProperty::Css, String::to_kuzu_type(), self.css.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Review {
    fn node_type(&self) -> NodeType {
        NodeType::Review
    }

    fn node_id(&self) -> NodeId {
        Review::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::ReviewAspect, String::to_kuzu_type(), self.options.review_aspect.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Section {
    fn node_type(&self) -> NodeType {
        NodeType::Section
    }

    fn node_id(&self) -> NodeId {
        Section::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::SectionType, String::to_kuzu_type(), self.section_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for SoftwareApplication {
    fn node_type(&self) -> NodeType {
        NodeType::SoftwareApplication
    }

    fn node_id(&self) -> NodeId {
        SoftwareApplication::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::SoftwareVersion, String::to_kuzu_type(), self.options.software_version.to_kuzu_value()),
            (NodeProperty::OperatingSystem, String::to_kuzu_type(), self.options.operating_system.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten())),
            (NodeProperty::SoftwareRequirements, relations(self.options.software_requirements.iter().flatten()))
        ]
    }
}

impl DatabaseNode for SoftwareSourceCode {
    fn node_type(&self) -> NodeType {
        NodeType::SoftwareSourceCode
    }

    fn node_id(&self) -> NodeId {
        SoftwareSourceCode::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::CodeRepository, String::to_kuzu_type(), self.code_repository.to_kuzu_value()),
            (NodeProperty::CodeSampleType, String::to_kuzu_type(), self.options.code_sample_type.to_kuzu_value()),
            (NodeProperty::RuntimePlatform, Vec::<String>::to_kuzu_type(), self.options.runtime_platform.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten())),
            (NodeProperty::TargetProducts, relations(self.target_products.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Strikeout {
    fn node_type(&self) -> NodeType {
        NodeType::Strikeout
    }

    fn node_id(&self) -> NodeId {
        Strikeout::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for Strong {
    fn node_type(&self) -> NodeType {
        NodeType::Strong
    }

    fn node_id(&self) -> NodeId {
        Strong::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for StyledBlock {
    fn node_type(&self) -> NodeType {
        NodeType::StyledBlock
    }

    fn node_id(&self) -> NodeId {
        StyledBlock::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::StyleLanguage, String::to_kuzu_type(), self.style_language.to_kuzu_value()),
            (NodeProperty::Css, String::to_kuzu_type(), self.options.css.to_kuzu_value()),
            (NodeProperty::ClassList, String::to_kuzu_type(), self.options.class_list.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for StyledInline {
    fn node_type(&self) -> NodeType {
        NodeType::StyledInline
    }

    fn node_id(&self) -> NodeId {
        StyledInline::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::StyleLanguage, String::to_kuzu_type(), self.style_language.to_kuzu_value()),
            (NodeProperty::Css, String::to_kuzu_type(), self.options.css.to_kuzu_value()),
            (NodeProperty::ClassList, String::to_kuzu_type(), self.options.class_list.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for Subscript {
    fn node_type(&self) -> NodeType {
        NodeType::Subscript
    }

    fn node_id(&self) -> NodeId {
        Subscript::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for Superscript {
    fn node_type(&self) -> NodeType {
        NodeType::Superscript
    }

    fn node_id(&self) -> NodeId {
        Superscript::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for Table {
    fn node_type(&self) -> NodeType {
        NodeType::Table
    }

    fn node_id(&self) -> NodeId {
        Table::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Label, String::to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, bool::to_kuzu_type(), self.label_automatically.to_kuzu_value()),
            (NodeProperty::Caption, String::to_kuzu_type(), to_text(&self.caption).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten())),
            (NodeProperty::Caption, relations(self.caption.iter().flatten())),
            (NodeProperty::Rows, relations(self.rows.iter())),
            (NodeProperty::Notes, relations(self.notes.iter().flatten()))
        ]
    }
}

impl DatabaseNode for TableCell {
    fn node_type(&self) -> NodeType {
        NodeType::TableCell
    }

    fn node_id(&self) -> NodeId {
        TableCell::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::CellType, String::to_kuzu_type(), self.cell_type.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::ColumnSpan, i64::to_kuzu_type(), self.options.column_span.to_kuzu_value()),
            (NodeProperty::RowSpan, i64::to_kuzu_type(), self.options.row_span.to_kuzu_value()),
            (NodeProperty::HorizontalAlignment, String::to_kuzu_type(), self.horizontal_alignment.to_kuzu_value()),
            (NodeProperty::HorizontalAlignmentCharacter, String::to_kuzu_type(), self.horizontal_alignment_character.to_kuzu_value()),
            (NodeProperty::VerticalAlignment, String::to_kuzu_type(), self.vertical_alignment.to_kuzu_value()),
            (NodeProperty::Text, String::to_kuzu_type(), to_text(self).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            
        ]
    }
}

impl DatabaseNode for TableRow {
    fn node_type(&self) -> NodeType {
        NodeType::TableRow
    }

    fn node_id(&self) -> NodeId {
        TableRow::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::RowType, String::to_kuzu_type(), self.row_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Cells, relations(self.cells.iter()))
        ]
    }
}

impl DatabaseNode for ThematicBreak {
    fn node_type(&self) -> NodeType {
        NodeType::ThematicBreak
    }

    fn node_id(&self) -> NodeId {
        ThematicBreak::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            
        ]
    }
}

impl DatabaseNode for Thing {
    fn node_type(&self) -> NodeType {
        NodeType::Thing
    }

    fn node_id(&self) -> NodeId {
        Thing::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Underline {
    fn node_type(&self) -> NodeType {
        NodeType::Underline
    }

    fn node_id(&self) -> NodeId {
        Underline::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
        ]
    }
}

impl DatabaseNode for Unknown {
    fn node_type(&self) -> NodeType {
        NodeType::Unknown
    }

    fn node_id(&self) -> NodeId {
        Unknown::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            
        ]
    }
}

impl DatabaseNode for Variable {
    fn node_type(&self) -> NodeType {
        NodeType::Variable
    }

    fn node_id(&self) -> NodeId {
        Variable::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::NativeType, String::to_kuzu_type(), self.native_type.to_kuzu_value()),
            (NodeProperty::NodeType, String::to_kuzu_type(), self.node_type.to_kuzu_value()),
            (NodeProperty::NativeHint, String::to_kuzu_type(), self.native_hint.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            
        ]
    }
}

impl DatabaseNode for VideoObject {
    fn node_type(&self) -> NodeType {
        NodeType::VideoObject
    }

    fn node_id(&self) -> NodeId {
        VideoObject::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, Vec::<String>::to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, String::to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, String::to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::DateCreated, Date::to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, Date::to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, Date::to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, Date::to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, Date::to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, Vec::<String>::to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, Vec::<String>::to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Bitrate, f64::to_kuzu_type(), self.options.bitrate.to_kuzu_value()),
            (NodeProperty::ContentSize, f64::to_kuzu_type(), self.options.content_size.to_kuzu_value()),
            (NodeProperty::ContentUrl, String::to_kuzu_type(), self.content_url.to_kuzu_value()),
            (NodeProperty::EmbedUrl, String::to_kuzu_type(), self.options.embed_url.to_kuzu_value()),
            (NodeProperty::MediaType, String::to_kuzu_type(), self.media_type.to_kuzu_value()),
            (NodeProperty::Transcript, String::to_kuzu_type(), self.options.transcript.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        vec![
            (NodeProperty::Images, relations(self.options.images.iter().flatten())),
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::Comments, relations(self.options.comments.iter().flatten())),
            (NodeProperty::Title, relations(self.title.iter().flatten())),
            (NodeProperty::Caption, relations(self.caption.iter().flatten())),
            (NodeProperty::Thumbnail, relations(self.options.thumbnail.iter()))
        ]
    }
}

#[allow(unreachable_patterns)]
impl DatabaseNode for Node {
    fn node_type(&self) -> NodeType {
        match self {
            Node::Admonition(node) => node.node_type(),
            Node::Annotation(node) => node.node_type(),
            Node::Article(node) => node.node_type(),
            Node::AudioObject(node) => node.node_type(),
            Node::AuthorRole(node) => node.node_type(),
            Node::Brand(node) => node.node_type(),
            Node::Cite(node) => node.node_type(),
            Node::CiteGroup(node) => node.node_type(),
            Node::Claim(node) => node.node_type(),
            Node::CodeBlock(node) => node.node_type(),
            Node::CodeChunk(node) => node.node_type(),
            Node::CodeExpression(node) => node.node_type(),
            Node::CodeInline(node) => node.node_type(),
            Node::Collection(node) => node.node_type(),
            Node::Comment(node) => node.node_type(),
            Node::ContactPoint(node) => node.node_type(),
            Node::CreativeWork(node) => node.node_type(),
            Node::DefinedTerm(node) => node.node_type(),
            Node::Directory(node) => node.node_type(),
            Node::Emphasis(node) => node.node_type(),
            Node::Figure(node) => node.node_type(),
            Node::File(node) => node.node_type(),
            Node::ForBlock(node) => node.node_type(),
            Node::Function(node) => node.node_type(),
            Node::Grant(node) => node.node_type(),
            Node::Heading(node) => node.node_type(),
            Node::IfBlock(node) => node.node_type(),
            Node::IfBlockClause(node) => node.node_type(),
            Node::ImageObject(node) => node.node_type(),
            Node::IncludeBlock(node) => node.node_type(),
            Node::Link(node) => node.node_type(),
            Node::List(node) => node.node_type(),
            Node::ListItem(node) => node.node_type(),
            Node::MathBlock(node) => node.node_type(),
            Node::MathInline(node) => node.node_type(),
            Node::MediaObject(node) => node.node_type(),
            Node::MonetaryGrant(node) => node.node_type(),
            Node::Note(node) => node.node_type(),
            Node::Organization(node) => node.node_type(),
            Node::Paragraph(node) => node.node_type(),
            Node::Parameter(node) => node.node_type(),
            Node::Periodical(node) => node.node_type(),
            Node::Person(node) => node.node_type(),
            Node::PostalAddress(node) => node.node_type(),
            Node::Product(node) => node.node_type(),
            Node::PropertyValue(node) => node.node_type(),
            Node::PublicationIssue(node) => node.node_type(),
            Node::PublicationVolume(node) => node.node_type(),
            Node::QuoteBlock(node) => node.node_type(),
            Node::QuoteInline(node) => node.node_type(),
            Node::RawBlock(node) => node.node_type(),
            Node::Review(node) => node.node_type(),
            Node::Section(node) => node.node_type(),
            Node::SoftwareApplication(node) => node.node_type(),
            Node::SoftwareSourceCode(node) => node.node_type(),
            Node::Strikeout(node) => node.node_type(),
            Node::Strong(node) => node.node_type(),
            Node::StyledBlock(node) => node.node_type(),
            Node::StyledInline(node) => node.node_type(),
            Node::Subscript(node) => node.node_type(),
            Node::Superscript(node) => node.node_type(),
            Node::Table(node) => node.node_type(),
            Node::TableCell(node) => node.node_type(),
            Node::TableRow(node) => node.node_type(),
            Node::ThematicBreak(node) => node.node_type(),
            Node::Thing(node) => node.node_type(),
            Node::Underline(node) => node.node_type(),
            Node::Unknown(node) => node.node_type(),
            Node::Variable(node) => node.node_type(),
            Node::VideoObject(node) => node.node_type(),
            _ => NodeType::Unknown
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Node::Admonition(node) => node.node_id(),
            Node::Annotation(node) => node.node_id(),
            Node::Article(node) => node.node_id(),
            Node::AudioObject(node) => node.node_id(),
            Node::AuthorRole(node) => node.node_id(),
            Node::Brand(node) => node.node_id(),
            Node::Cite(node) => node.node_id(),
            Node::CiteGroup(node) => node.node_id(),
            Node::Claim(node) => node.node_id(),
            Node::CodeBlock(node) => node.node_id(),
            Node::CodeChunk(node) => node.node_id(),
            Node::CodeExpression(node) => node.node_id(),
            Node::CodeInline(node) => node.node_id(),
            Node::Collection(node) => node.node_id(),
            Node::Comment(node) => node.node_id(),
            Node::ContactPoint(node) => node.node_id(),
            Node::CreativeWork(node) => node.node_id(),
            Node::DefinedTerm(node) => node.node_id(),
            Node::Directory(node) => node.node_id(),
            Node::Emphasis(node) => node.node_id(),
            Node::Figure(node) => node.node_id(),
            Node::File(node) => node.node_id(),
            Node::ForBlock(node) => node.node_id(),
            Node::Function(node) => node.node_id(),
            Node::Grant(node) => node.node_id(),
            Node::Heading(node) => node.node_id(),
            Node::IfBlock(node) => node.node_id(),
            Node::IfBlockClause(node) => node.node_id(),
            Node::ImageObject(node) => node.node_id(),
            Node::IncludeBlock(node) => node.node_id(),
            Node::Link(node) => node.node_id(),
            Node::List(node) => node.node_id(),
            Node::ListItem(node) => node.node_id(),
            Node::MathBlock(node) => node.node_id(),
            Node::MathInline(node) => node.node_id(),
            Node::MediaObject(node) => node.node_id(),
            Node::MonetaryGrant(node) => node.node_id(),
            Node::Note(node) => node.node_id(),
            Node::Organization(node) => node.node_id(),
            Node::Paragraph(node) => node.node_id(),
            Node::Parameter(node) => node.node_id(),
            Node::Periodical(node) => node.node_id(),
            Node::Person(node) => node.node_id(),
            Node::PostalAddress(node) => node.node_id(),
            Node::Product(node) => node.node_id(),
            Node::PropertyValue(node) => node.node_id(),
            Node::PublicationIssue(node) => node.node_id(),
            Node::PublicationVolume(node) => node.node_id(),
            Node::QuoteBlock(node) => node.node_id(),
            Node::QuoteInline(node) => node.node_id(),
            Node::RawBlock(node) => node.node_id(),
            Node::Review(node) => node.node_id(),
            Node::Section(node) => node.node_id(),
            Node::SoftwareApplication(node) => node.node_id(),
            Node::SoftwareSourceCode(node) => node.node_id(),
            Node::Strikeout(node) => node.node_id(),
            Node::Strong(node) => node.node_id(),
            Node::StyledBlock(node) => node.node_id(),
            Node::StyledInline(node) => node.node_id(),
            Node::Subscript(node) => node.node_id(),
            Node::Superscript(node) => node.node_id(),
            Node::Table(node) => node.node_id(),
            Node::TableCell(node) => node.node_id(),
            Node::TableRow(node) => node.node_id(),
            Node::ThematicBreak(node) => node.node_id(),
            Node::Thing(node) => node.node_id(),
            Node::Underline(node) => node.node_id(),
            Node::Unknown(node) => node.node_id(),
            Node::Variable(node) => node.node_id(),
            Node::VideoObject(node) => node.node_id(),
            _ => NodeId::null()
        }
    }

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        match self {
            Node::Admonition(node) => node.node_table(),
            Node::Annotation(node) => node.node_table(),
            Node::Article(node) => node.node_table(),
            Node::AudioObject(node) => node.node_table(),
            Node::AuthorRole(node) => node.node_table(),
            Node::Brand(node) => node.node_table(),
            Node::Cite(node) => node.node_table(),
            Node::CiteGroup(node) => node.node_table(),
            Node::Claim(node) => node.node_table(),
            Node::CodeBlock(node) => node.node_table(),
            Node::CodeChunk(node) => node.node_table(),
            Node::CodeExpression(node) => node.node_table(),
            Node::CodeInline(node) => node.node_table(),
            Node::Collection(node) => node.node_table(),
            Node::Comment(node) => node.node_table(),
            Node::ContactPoint(node) => node.node_table(),
            Node::CreativeWork(node) => node.node_table(),
            Node::DefinedTerm(node) => node.node_table(),
            Node::Directory(node) => node.node_table(),
            Node::Emphasis(node) => node.node_table(),
            Node::Figure(node) => node.node_table(),
            Node::File(node) => node.node_table(),
            Node::ForBlock(node) => node.node_table(),
            Node::Function(node) => node.node_table(),
            Node::Grant(node) => node.node_table(),
            Node::Heading(node) => node.node_table(),
            Node::IfBlock(node) => node.node_table(),
            Node::IfBlockClause(node) => node.node_table(),
            Node::ImageObject(node) => node.node_table(),
            Node::IncludeBlock(node) => node.node_table(),
            Node::Link(node) => node.node_table(),
            Node::List(node) => node.node_table(),
            Node::ListItem(node) => node.node_table(),
            Node::MathBlock(node) => node.node_table(),
            Node::MathInline(node) => node.node_table(),
            Node::MediaObject(node) => node.node_table(),
            Node::MonetaryGrant(node) => node.node_table(),
            Node::Note(node) => node.node_table(),
            Node::Organization(node) => node.node_table(),
            Node::Paragraph(node) => node.node_table(),
            Node::Parameter(node) => node.node_table(),
            Node::Periodical(node) => node.node_table(),
            Node::Person(node) => node.node_table(),
            Node::PostalAddress(node) => node.node_table(),
            Node::Product(node) => node.node_table(),
            Node::PropertyValue(node) => node.node_table(),
            Node::PublicationIssue(node) => node.node_table(),
            Node::PublicationVolume(node) => node.node_table(),
            Node::QuoteBlock(node) => node.node_table(),
            Node::QuoteInline(node) => node.node_table(),
            Node::RawBlock(node) => node.node_table(),
            Node::Review(node) => node.node_table(),
            Node::Section(node) => node.node_table(),
            Node::SoftwareApplication(node) => node.node_table(),
            Node::SoftwareSourceCode(node) => node.node_table(),
            Node::Strikeout(node) => node.node_table(),
            Node::Strong(node) => node.node_table(),
            Node::StyledBlock(node) => node.node_table(),
            Node::StyledInline(node) => node.node_table(),
            Node::Subscript(node) => node.node_table(),
            Node::Superscript(node) => node.node_table(),
            Node::Table(node) => node.node_table(),
            Node::TableCell(node) => node.node_table(),
            Node::TableRow(node) => node.node_table(),
            Node::ThematicBreak(node) => node.node_table(),
            Node::Thing(node) => node.node_table(),
            Node::Underline(node) => node.node_table(),
            Node::Unknown(node) => node.node_table(),
            Node::Variable(node) => node.node_table(),
            Node::VideoObject(node) => node.node_table(),
            _ => Vec::new()
        }
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        match self {
            Node::Admonition(node) => node.rel_tables(),
            Node::Annotation(node) => node.rel_tables(),
            Node::Article(node) => node.rel_tables(),
            Node::AudioObject(node) => node.rel_tables(),
            Node::AuthorRole(node) => node.rel_tables(),
            Node::Brand(node) => node.rel_tables(),
            Node::Cite(node) => node.rel_tables(),
            Node::CiteGroup(node) => node.rel_tables(),
            Node::Claim(node) => node.rel_tables(),
            Node::CodeBlock(node) => node.rel_tables(),
            Node::CodeChunk(node) => node.rel_tables(),
            Node::CodeExpression(node) => node.rel_tables(),
            Node::CodeInline(node) => node.rel_tables(),
            Node::Collection(node) => node.rel_tables(),
            Node::Comment(node) => node.rel_tables(),
            Node::ContactPoint(node) => node.rel_tables(),
            Node::CreativeWork(node) => node.rel_tables(),
            Node::DefinedTerm(node) => node.rel_tables(),
            Node::Directory(node) => node.rel_tables(),
            Node::Emphasis(node) => node.rel_tables(),
            Node::Figure(node) => node.rel_tables(),
            Node::File(node) => node.rel_tables(),
            Node::ForBlock(node) => node.rel_tables(),
            Node::Function(node) => node.rel_tables(),
            Node::Grant(node) => node.rel_tables(),
            Node::Heading(node) => node.rel_tables(),
            Node::IfBlock(node) => node.rel_tables(),
            Node::IfBlockClause(node) => node.rel_tables(),
            Node::ImageObject(node) => node.rel_tables(),
            Node::IncludeBlock(node) => node.rel_tables(),
            Node::Link(node) => node.rel_tables(),
            Node::List(node) => node.rel_tables(),
            Node::ListItem(node) => node.rel_tables(),
            Node::MathBlock(node) => node.rel_tables(),
            Node::MathInline(node) => node.rel_tables(),
            Node::MediaObject(node) => node.rel_tables(),
            Node::MonetaryGrant(node) => node.rel_tables(),
            Node::Note(node) => node.rel_tables(),
            Node::Organization(node) => node.rel_tables(),
            Node::Paragraph(node) => node.rel_tables(),
            Node::Parameter(node) => node.rel_tables(),
            Node::Periodical(node) => node.rel_tables(),
            Node::Person(node) => node.rel_tables(),
            Node::PostalAddress(node) => node.rel_tables(),
            Node::Product(node) => node.rel_tables(),
            Node::PropertyValue(node) => node.rel_tables(),
            Node::PublicationIssue(node) => node.rel_tables(),
            Node::PublicationVolume(node) => node.rel_tables(),
            Node::QuoteBlock(node) => node.rel_tables(),
            Node::QuoteInline(node) => node.rel_tables(),
            Node::RawBlock(node) => node.rel_tables(),
            Node::Review(node) => node.rel_tables(),
            Node::Section(node) => node.rel_tables(),
            Node::SoftwareApplication(node) => node.rel_tables(),
            Node::SoftwareSourceCode(node) => node.rel_tables(),
            Node::Strikeout(node) => node.rel_tables(),
            Node::Strong(node) => node.rel_tables(),
            Node::StyledBlock(node) => node.rel_tables(),
            Node::StyledInline(node) => node.rel_tables(),
            Node::Subscript(node) => node.rel_tables(),
            Node::Superscript(node) => node.rel_tables(),
            Node::Table(node) => node.rel_tables(),
            Node::TableCell(node) => node.rel_tables(),
            Node::TableRow(node) => node.rel_tables(),
            Node::ThematicBreak(node) => node.rel_tables(),
            Node::Thing(node) => node.rel_tables(),
            Node::Underline(node) => node.rel_tables(),
            Node::Unknown(node) => node.rel_tables(),
            Node::Variable(node) => node.rel_tables(),
            Node::VideoObject(node) => node.rel_tables(),
            _ => Vec::new()
        }
    }
}

#[allow(unreachable_patterns)]
impl DatabaseNode for Block {
    fn node_type(&self) -> NodeType {
        match self {
            Block::Admonition(node) => node.node_type(),
            Block::AudioObject(node) => node.node_type(),
            Block::Claim(node) => node.node_type(),
            Block::CodeBlock(node) => node.node_type(),
            Block::CodeChunk(node) => node.node_type(),
            Block::Figure(node) => node.node_type(),
            Block::File(node) => node.node_type(),
            Block::ForBlock(node) => node.node_type(),
            Block::Heading(node) => node.node_type(),
            Block::IfBlock(node) => node.node_type(),
            Block::ImageObject(node) => node.node_type(),
            Block::IncludeBlock(node) => node.node_type(),
            Block::List(node) => node.node_type(),
            Block::MathBlock(node) => node.node_type(),
            Block::Paragraph(node) => node.node_type(),
            Block::QuoteBlock(node) => node.node_type(),
            Block::RawBlock(node) => node.node_type(),
            Block::Section(node) => node.node_type(),
            Block::StyledBlock(node) => node.node_type(),
            Block::Table(node) => node.node_type(),
            Block::ThematicBreak(node) => node.node_type(),
            Block::VideoObject(node) => node.node_type(),
            _ => NodeType::Unknown
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Block::Admonition(node) => node.node_id(),
            Block::AudioObject(node) => node.node_id(),
            Block::Claim(node) => node.node_id(),
            Block::CodeBlock(node) => node.node_id(),
            Block::CodeChunk(node) => node.node_id(),
            Block::Figure(node) => node.node_id(),
            Block::File(node) => node.node_id(),
            Block::ForBlock(node) => node.node_id(),
            Block::Heading(node) => node.node_id(),
            Block::IfBlock(node) => node.node_id(),
            Block::ImageObject(node) => node.node_id(),
            Block::IncludeBlock(node) => node.node_id(),
            Block::List(node) => node.node_id(),
            Block::MathBlock(node) => node.node_id(),
            Block::Paragraph(node) => node.node_id(),
            Block::QuoteBlock(node) => node.node_id(),
            Block::RawBlock(node) => node.node_id(),
            Block::Section(node) => node.node_id(),
            Block::StyledBlock(node) => node.node_id(),
            Block::Table(node) => node.node_id(),
            Block::ThematicBreak(node) => node.node_id(),
            Block::VideoObject(node) => node.node_id(),
            _ => NodeId::null()
        }
    }

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        match self {
            Block::Admonition(node) => node.node_table(),
            Block::AudioObject(node) => node.node_table(),
            Block::Claim(node) => node.node_table(),
            Block::CodeBlock(node) => node.node_table(),
            Block::CodeChunk(node) => node.node_table(),
            Block::Figure(node) => node.node_table(),
            Block::File(node) => node.node_table(),
            Block::ForBlock(node) => node.node_table(),
            Block::Heading(node) => node.node_table(),
            Block::IfBlock(node) => node.node_table(),
            Block::ImageObject(node) => node.node_table(),
            Block::IncludeBlock(node) => node.node_table(),
            Block::List(node) => node.node_table(),
            Block::MathBlock(node) => node.node_table(),
            Block::Paragraph(node) => node.node_table(),
            Block::QuoteBlock(node) => node.node_table(),
            Block::RawBlock(node) => node.node_table(),
            Block::Section(node) => node.node_table(),
            Block::StyledBlock(node) => node.node_table(),
            Block::Table(node) => node.node_table(),
            Block::ThematicBreak(node) => node.node_table(),
            Block::VideoObject(node) => node.node_table(),
            _ => Vec::new()
        }
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        match self {
            Block::Admonition(node) => node.rel_tables(),
            Block::AudioObject(node) => node.rel_tables(),
            Block::Claim(node) => node.rel_tables(),
            Block::CodeBlock(node) => node.rel_tables(),
            Block::CodeChunk(node) => node.rel_tables(),
            Block::Figure(node) => node.rel_tables(),
            Block::File(node) => node.rel_tables(),
            Block::ForBlock(node) => node.rel_tables(),
            Block::Heading(node) => node.rel_tables(),
            Block::IfBlock(node) => node.rel_tables(),
            Block::ImageObject(node) => node.rel_tables(),
            Block::IncludeBlock(node) => node.rel_tables(),
            Block::List(node) => node.rel_tables(),
            Block::MathBlock(node) => node.rel_tables(),
            Block::Paragraph(node) => node.rel_tables(),
            Block::QuoteBlock(node) => node.rel_tables(),
            Block::RawBlock(node) => node.rel_tables(),
            Block::Section(node) => node.rel_tables(),
            Block::StyledBlock(node) => node.rel_tables(),
            Block::Table(node) => node.rel_tables(),
            Block::ThematicBreak(node) => node.rel_tables(),
            Block::VideoObject(node) => node.rel_tables(),
            _ => Vec::new()
        }
    }
}

#[allow(unreachable_patterns)]
impl DatabaseNode for Inline {
    fn node_type(&self) -> NodeType {
        match self {
            Inline::Annotation(node) => node.node_type(),
            Inline::AudioObject(node) => node.node_type(),
            Inline::Cite(node) => node.node_type(),
            Inline::CiteGroup(node) => node.node_type(),
            Inline::CodeExpression(node) => node.node_type(),
            Inline::CodeInline(node) => node.node_type(),
            Inline::Emphasis(node) => node.node_type(),
            Inline::ImageObject(node) => node.node_type(),
            Inline::Link(node) => node.node_type(),
            Inline::MathInline(node) => node.node_type(),
            Inline::MediaObject(node) => node.node_type(),
            Inline::Note(node) => node.node_type(),
            Inline::Parameter(node) => node.node_type(),
            Inline::QuoteInline(node) => node.node_type(),
            Inline::StyledInline(node) => node.node_type(),
            Inline::Strikeout(node) => node.node_type(),
            Inline::Strong(node) => node.node_type(),
            Inline::Subscript(node) => node.node_type(),
            Inline::Superscript(node) => node.node_type(),
            Inline::Underline(node) => node.node_type(),
            Inline::VideoObject(node) => node.node_type(),
            _ => NodeType::Unknown
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Inline::Annotation(node) => node.node_id(),
            Inline::AudioObject(node) => node.node_id(),
            Inline::Cite(node) => node.node_id(),
            Inline::CiteGroup(node) => node.node_id(),
            Inline::CodeExpression(node) => node.node_id(),
            Inline::CodeInline(node) => node.node_id(),
            Inline::Emphasis(node) => node.node_id(),
            Inline::ImageObject(node) => node.node_id(),
            Inline::Link(node) => node.node_id(),
            Inline::MathInline(node) => node.node_id(),
            Inline::MediaObject(node) => node.node_id(),
            Inline::Note(node) => node.node_id(),
            Inline::Parameter(node) => node.node_id(),
            Inline::QuoteInline(node) => node.node_id(),
            Inline::StyledInline(node) => node.node_id(),
            Inline::Strikeout(node) => node.node_id(),
            Inline::Strong(node) => node.node_id(),
            Inline::Subscript(node) => node.node_id(),
            Inline::Superscript(node) => node.node_id(),
            Inline::Underline(node) => node.node_id(),
            Inline::VideoObject(node) => node.node_id(),
            _ => NodeId::null()
        }
    }

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        match self {
            Inline::Annotation(node) => node.node_table(),
            Inline::AudioObject(node) => node.node_table(),
            Inline::Cite(node) => node.node_table(),
            Inline::CiteGroup(node) => node.node_table(),
            Inline::CodeExpression(node) => node.node_table(),
            Inline::CodeInline(node) => node.node_table(),
            Inline::Emphasis(node) => node.node_table(),
            Inline::ImageObject(node) => node.node_table(),
            Inline::Link(node) => node.node_table(),
            Inline::MathInline(node) => node.node_table(),
            Inline::MediaObject(node) => node.node_table(),
            Inline::Note(node) => node.node_table(),
            Inline::Parameter(node) => node.node_table(),
            Inline::QuoteInline(node) => node.node_table(),
            Inline::StyledInline(node) => node.node_table(),
            Inline::Strikeout(node) => node.node_table(),
            Inline::Strong(node) => node.node_table(),
            Inline::Subscript(node) => node.node_table(),
            Inline::Superscript(node) => node.node_table(),
            Inline::Underline(node) => node.node_table(),
            Inline::VideoObject(node) => node.node_table(),
            _ => Vec::new()
        }
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        match self {
            Inline::Annotation(node) => node.rel_tables(),
            Inline::AudioObject(node) => node.rel_tables(),
            Inline::Cite(node) => node.rel_tables(),
            Inline::CiteGroup(node) => node.rel_tables(),
            Inline::CodeExpression(node) => node.rel_tables(),
            Inline::CodeInline(node) => node.rel_tables(),
            Inline::Emphasis(node) => node.rel_tables(),
            Inline::ImageObject(node) => node.rel_tables(),
            Inline::Link(node) => node.rel_tables(),
            Inline::MathInline(node) => node.rel_tables(),
            Inline::MediaObject(node) => node.rel_tables(),
            Inline::Note(node) => node.rel_tables(),
            Inline::Parameter(node) => node.rel_tables(),
            Inline::QuoteInline(node) => node.rel_tables(),
            Inline::StyledInline(node) => node.rel_tables(),
            Inline::Strikeout(node) => node.rel_tables(),
            Inline::Strong(node) => node.rel_tables(),
            Inline::Subscript(node) => node.rel_tables(),
            Inline::Superscript(node) => node.rel_tables(),
            Inline::Underline(node) => node.rel_tables(),
            Inline::VideoObject(node) => node.rel_tables(),
            _ => Vec::new()
        }
    }
}

#[allow(unreachable_patterns)]
impl DatabaseNode for Author {
    fn node_type(&self) -> NodeType {
        match self {
            Author::Person(node) => node.node_type(),
            Author::Organization(node) => node.node_type(),
            Author::SoftwareApplication(node) => node.node_type(),
            Author::AuthorRole(node) => node.node_type(),
            _ => NodeType::Unknown
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Author::Person(node) => node.node_id(),
            Author::Organization(node) => node.node_id(),
            Author::SoftwareApplication(node) => node.node_id(),
            Author::AuthorRole(node) => node.node_id(),
            _ => NodeId::null()
        }
    }

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        match self {
            Author::Person(node) => node.node_table(),
            Author::Organization(node) => node.node_table(),
            Author::SoftwareApplication(node) => node.node_table(),
            Author::AuthorRole(node) => node.node_table(),
            _ => Vec::new()
        }
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)> {
        match self {
            Author::Person(node) => node.rel_tables(),
            Author::Organization(node) => node.rel_tables(),
            Author::SoftwareApplication(node) => node.rel_tables(),
            Author::AuthorRole(node) => node.rel_tables(),
            _ => Vec::new()
        }
    }
}

