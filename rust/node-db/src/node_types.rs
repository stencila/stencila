// Generated file; do not edit. See the Rust `schema-gen` crate.

use kuzu::{LogicalType, Value};

use schema::*;

use super::{DatabaseNode, ToKuzu};

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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Title, self.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Annotation, self.annotation.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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
            (NodeProperty::ExecutionInstance, String::to_kuzu_type(), self.options.execution_instance.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Pagination, String::to_kuzu_type(), self.options.pagination.to_kuzu_value()),
            (NodeProperty::Frontmatter, String::to_kuzu_type(), self.frontmatter.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Caption, self.caption.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Author, vec![(self.author.node_type(), self.author.node_id(), 1)])
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Logo, self.options.logo.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            (NodeProperty::CitationIntent, String::to_kuzu_type(), self.options.citation_intent.to_kuzu_value()),
            (NodeProperty::Pagination, String::to_kuzu_type(), self.options.pagination.to_kuzu_value()),
            (NodeProperty::CitationPrefix, String::to_kuzu_type(), self.options.citation_prefix.to_kuzu_value()),
            (NodeProperty::CitationSuffix, String::to_kuzu_type(), self.options.citation_suffix.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.options.content.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Items, self.items.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            (NodeProperty::ExecutionInstance, String::to_kuzu_type(), self.options.execution_instance.to_kuzu_value()),
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
            (NodeProperty::ExecutionPure, bool::to_kuzu_type(), self.options.execution_pure.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Caption, self.caption.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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
            (NodeProperty::ExecutionInstance, String::to_kuzu_type(), self.options.execution_instance.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, String::to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, String::to_kuzu_type(), self.options.execution_bounded.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::ParentItem, self.options.parent_item.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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
            (NodeProperty::LabelAutomatically, bool::to_kuzu_type(), self.label_automatically.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Caption, self.caption.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
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
            (NodeProperty::ExecutionInstance, String::to_kuzu_type(), self.options.execution_instance.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, String::to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, String::to_kuzu_type(), self.options.execution_bounded.to_kuzu_value()),
            (NodeProperty::Variable, String::to_kuzu_type(), self.variable.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Otherwise, self.otherwise.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Iterations, self.iterations.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Parameters, self.parameters.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            (NodeProperty::ExecutionInstance, String::to_kuzu_type(), self.options.execution_instance.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Clauses, self.clauses.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            (NodeProperty::ExecutionInstance, String::to_kuzu_type(), self.options.execution_instance.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, String::to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, String::to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, String::to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, String::to_kuzu_type(), self.options.execution_bounded.to_kuzu_value()),
            (NodeProperty::IsActive, bool::to_kuzu_type(), self.is_active.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Caption, self.caption.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Thumbnail, self.options.thumbnail.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            (NodeProperty::ExecutionInstance, String::to_kuzu_type(), self.options.execution_instance.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Source, String::to_kuzu_type(), self.source.to_kuzu_value()),
            (NodeProperty::MediaType, String::to_kuzu_type(), self.media_type.to_kuzu_value()),
            (NodeProperty::Select, String::to_kuzu_type(), self.select.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Items, self.items.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            (NodeProperty::IsChecked, bool::to_kuzu_type(), self.is_checked.to_kuzu_value()),
            (NodeProperty::Position, i64::to_kuzu_type(), self.position.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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
            (NodeProperty::Mathml, String::to_kuzu_type(), self.options.mathml.to_kuzu_value()),
            (NodeProperty::Label, String::to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, bool::to_kuzu_type(), self.label_automatically.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            (NodeProperty::MathLanguage, String::to_kuzu_type(), self.math_language.to_kuzu_value()),
            (NodeProperty::Mathml, String::to_kuzu_type(), self.options.mathml.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Brands, self.options.brands.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::ContactPoints, self.options.contact_points.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Departments, self.options.departments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Logo, self.options.logo.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::ParentOrganization, self.options.parent_organization.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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
            (NodeProperty::ExecutionInstance, String::to_kuzu_type(), self.options.execution_instance.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, Timestamp::to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, Duration::to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Name, String::to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Label, String::to_kuzu_type(), self.options.label.to_kuzu_value()),
            (NodeProperty::DerivedFrom, String::to_kuzu_type(), self.options.derived_from.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Affiliations, self.affiliations.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::MemberOf, self.options.member_of.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Brands, self.options.brands.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Logo, self.options.logo.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::SoftwareRequirements, self.options.software_requirements.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::TargetProducts, self.target_products.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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
            (NodeProperty::LabelAutomatically, bool::to_kuzu_type(), self.label_automatically.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.options.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Caption, self.caption.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Rows, self.rows.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Notes, self.notes.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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
            (NodeProperty::VerticalAlignment, String::to_kuzu_type(), self.vertical_alignment.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Cells, self.cells.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
        ]
    }
}

impl DatabaseNode for Text {
    fn node_type(&self) -> NodeType {
        NodeType::Text
    }

    fn node_id(&self) -> NodeId {
        Text::node_id(self)
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Value, String::to_kuzu_type(), self.value.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Content, self.content.iter().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect())
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)> {
        vec![
            (NodeProperty::Images, self.options.images.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Abstract, self.options.r#abstract.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Authors, self.options.authors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Contributors, self.options.contributors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Editors, self.options.editors.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Comments, self.options.comments.iter().flatten().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect()),
            (NodeProperty::Title, self.title.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Caption, self.caption.iter().flatten().enumerate().flat_map(|(index, item)| item.node_id().map(|node_id| (item.node_type(), node_id, index + 1))).collect()),
            (NodeProperty::Thumbnail, self.options.thumbnail.iter().enumerate().map(|(index, item)| (item.node_type(), item.node_id(), index + 1)).collect())
        ]
    }
}

