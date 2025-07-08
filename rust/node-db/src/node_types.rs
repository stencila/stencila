// Generated file, do not edit. See the Rust `schema-gen` crate.

use kernel_kuzu::{kuzu::{LogicalType, Value}, ToKuzu};
use codec_text_trait::to_text;
use schema::*;

use super::{embeddings_property, embeddings_type, DatabaseNode};

pub(super) fn primary_key(node_type: &NodeType) -> &'static str {
    match node_type {
        NodeType::Organization => "ror",
        NodeType::Person => "orcid",
        NodeType::Reference => "doi",
        _ => "nodeId"
    }
}

fn relations<'lt, I, D>(iter: I) -> Vec<(NodeType, Value)>
where
    I: Iterator<Item = &'lt D>,
    D: DatabaseNode + 'lt,
{
    iter.flat_map(|item| (!matches!(item.node_type(), NodeType::Unknown)).then_some((item.node_type(), item.primary_key())))
        .collect()
}

impl DatabaseNode for Admonition {
    fn node_type(&self) -> NodeType {
        NodeType::Admonition
    }

    fn node_id(&self) -> NodeId {
        Admonition::node_id(self)
    }
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AdmonitionType, self.admonition_type.to_kuzu_type(), self.admonition_type.to_kuzu_value()),
            (NodeProperty::IsFolded, self.is_folded.to_kuzu_type(), self.is_folded.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.description.to_kuzu_type(), self.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.date_created.to_kuzu_type(), self.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.date_received.to_kuzu_type(), self.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.date_accepted.to_kuzu_type(), self.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.date_modified.to_kuzu_type(), self.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.date_published.to_kuzu_type(), self.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.genre.to_kuzu_type(), self.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.keywords.to_kuzu_type(), self.keywords.to_kuzu_value()),
            (NodeProperty::ExecutionMode, self.execution_mode.to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, self.options.execution_count.to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, self.options.execution_required.to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, self.options.execution_status.to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, self.options.execution_ended.to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, self.options.execution_duration.to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Pagination, self.options.pagination.to_kuzu_type(), self.options.pagination.to_kuzu_value()),
            (NodeProperty::Frontmatter, self.frontmatter.to_kuzu_type(), self.frontmatter.to_kuzu_value()),
            (NodeProperty::Repository, self.options.repository.to_kuzu_type(), self.options.repository.to_kuzu_value()),
            (NodeProperty::Path, self.options.path.to_kuzu_type(), self.options.path.to_kuzu_value()),
            (NodeProperty::Title, LogicalType::String, to_text(&self.title).to_kuzu_value()),
            (NodeProperty::Abstract, LogicalType::String, to_text(&self.r#abstract).to_kuzu_value()),
            (embeddings_property(), embeddings_type(), Null.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.references.iter().flatten())),
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.options.date_created.to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.options.date_received.to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.options.date_accepted.to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.options.date_modified.to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.options.date_published.to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Bitrate, self.options.bitrate.to_kuzu_type(), self.options.bitrate.to_kuzu_value()),
            (NodeProperty::ContentSize, self.options.content_size.to_kuzu_type(), self.options.content_size.to_kuzu_value()),
            (NodeProperty::ContentUrl, self.content_url.to_kuzu_type(), self.content_url.to_kuzu_value()),
            (NodeProperty::EmbedUrl, self.options.embed_url.to_kuzu_type(), self.options.embed_url.to_kuzu_value()),
            (NodeProperty::MediaType, self.media_type.to_kuzu_type(), self.media_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.options.references.iter().flatten())),
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::RoleName, self.role_name.to_kuzu_type(), self.role_name.to_kuzu_value()),
            (NodeProperty::Format, self.format.to_kuzu_type(), self.format.to_kuzu_value()),
            (NodeProperty::LastModified, self.last_modified.to_kuzu_type(), self.last_modified.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Author, vec![(self.author.node_type(), self.author.primary_key())])
        ]
    }
}

impl DatabaseNode for Citation {
    fn node_type(&self) -> NodeType {
        NodeType::Citation
    }

    fn node_id(&self) -> NodeId {
        Citation::node_id(self)
    }
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Target, self.target.to_kuzu_type(), self.target.to_kuzu_value()),
            (NodeProperty::CitationMode, self.citation_mode.to_kuzu_type(), self.citation_mode.to_kuzu_value()),
            (NodeProperty::CitationIntent, self.options.citation_intent.to_kuzu_type(), self.options.citation_intent.to_kuzu_value()),
            (NodeProperty::Text, LogicalType::String, to_text(&self.options.content).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Cites, relations(self.options.cites.iter()))
        ]
    }
}

impl DatabaseNode for CitationGroup {
    fn node_type(&self) -> NodeType {
        NodeType::CitationGroup
    }

    fn node_id(&self) -> NodeId {
        CitationGroup::node_id(self)
    }
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.options.date_created.to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.options.date_received.to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.options.date_accepted.to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.options.date_modified.to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.options.date_published.to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::ClaimType, self.claim_type.to_kuzu_type(), self.claim_type.to_kuzu_value()),
            (NodeProperty::Label, self.label.to_kuzu_type(), self.label.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.options.references.iter().flatten())),
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, self.programming_language.to_kuzu_type(), self.programming_language.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, self.execution_mode.to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, self.options.execution_count.to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, self.options.execution_required.to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, self.options.execution_status.to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, self.options.execution_ended.to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, self.options.execution_duration.to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, self.programming_language.to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, self.execution_bounds.to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, self.options.execution_bounded.to_kuzu_type(), self.options.execution_bounded.to_kuzu_value()),
            (NodeProperty::LabelType, self.label_type.to_kuzu_type(), self.label_type.to_kuzu_value()),
            (NodeProperty::Label, self.label.to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, self.label_automatically.to_kuzu_type(), self.label_automatically.to_kuzu_value()),
            (NodeProperty::IsEchoed, self.is_echoed.to_kuzu_type(), self.is_echoed.to_kuzu_value()),
            (NodeProperty::IsHidden, self.is_hidden.to_kuzu_type(), self.is_hidden.to_kuzu_value()),
            (NodeProperty::ExecutionPure, self.options.execution_pure.to_kuzu_type(), self.options.execution_pure.to_kuzu_value()),
            (NodeProperty::Caption, LogicalType::String, to_text(&self.caption).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, self.execution_mode.to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, self.options.execution_count.to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, self.options.execution_required.to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, self.options.execution_status.to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, self.options.execution_ended.to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, self.options.execution_duration.to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, self.programming_language.to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, self.execution_bounds.to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, self.options.execution_bounded.to_kuzu_type(), self.options.execution_bounded.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Name, self.name.to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Path, self.path.to_kuzu_type(), self.path.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.options.date_created.to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.options.date_received.to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.options.date_accepted.to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.options.date_modified.to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.options.date_published.to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Label, self.label.to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, self.label_automatically.to_kuzu_type(), self.label_automatically.to_kuzu_value()),
            (NodeProperty::Caption, LogicalType::String, to_text(&self.caption).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.options.references.iter().flatten())),
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Name, self.name.to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Path, self.path.to_kuzu_type(), self.path.to_kuzu_value()),
            (NodeProperty::MediaType, self.media_type.to_kuzu_type(), self.media_type.to_kuzu_value()),
            (NodeProperty::TransferEncoding, self.options.transfer_encoding.to_kuzu_type(), self.options.transfer_encoding.to_kuzu_value()),
            (NodeProperty::Size, self.size.to_kuzu_type(), self.size.to_kuzu_value()),
            (NodeProperty::Content, self.content.to_kuzu_type(), self.content.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, self.execution_mode.to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, self.options.execution_count.to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, self.options.execution_required.to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, self.options.execution_status.to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, self.options.execution_ended.to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, self.options.execution_duration.to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, self.programming_language.to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, self.execution_bounds.to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, self.options.execution_bounded.to_kuzu_type(), self.options.execution_bounded.to_kuzu_value()),
            (NodeProperty::Variable, self.variable.to_kuzu_type(), self.variable.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Name, self.name.to_kuzu_type(), self.name.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Parameters, relations(self.parameters.iter()))
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::LabelType, self.label_type.to_kuzu_type(), self.label_type.to_kuzu_value()),
            (NodeProperty::Label, self.label.to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::Level, self.level.to_kuzu_type(), self.level.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, self.execution_mode.to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, self.options.execution_count.to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, self.options.execution_required.to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, self.options.execution_status.to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, self.options.execution_ended.to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, self.options.execution_duration.to_kuzu_type(), self.options.execution_duration.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, self.execution_mode.to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, self.options.execution_count.to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, self.options.execution_required.to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, self.options.execution_status.to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, self.options.execution_ended.to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, self.options.execution_duration.to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, self.programming_language.to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::ExecutionBounds, self.execution_bounds.to_kuzu_type(), self.execution_bounds.to_kuzu_value()),
            (NodeProperty::ExecutionBounded, self.options.execution_bounded.to_kuzu_type(), self.options.execution_bounded.to_kuzu_value()),
            (NodeProperty::IsActive, self.is_active.to_kuzu_type(), self.is_active.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.options.date_created.to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.options.date_received.to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.options.date_accepted.to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.options.date_modified.to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.options.date_published.to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Bitrate, self.options.bitrate.to_kuzu_type(), self.options.bitrate.to_kuzu_value()),
            (NodeProperty::ContentSize, self.options.content_size.to_kuzu_type(), self.options.content_size.to_kuzu_value()),
            (NodeProperty::ContentUrl, self.content_url.to_kuzu_type(), self.content_url.to_kuzu_value()),
            (NodeProperty::EmbedUrl, self.options.embed_url.to_kuzu_type(), self.options.embed_url.to_kuzu_value()),
            (NodeProperty::MediaType, self.media_type.to_kuzu_type(), self.media_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.options.references.iter().flatten())),
            (NodeProperty::Title, relations(self.title.iter().flatten())),
            (NodeProperty::Caption, relations(self.caption.iter().flatten()))
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, self.execution_mode.to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, self.options.execution_count.to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, self.options.execution_required.to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, self.options.execution_status.to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, self.options.execution_ended.to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, self.options.execution_duration.to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Source, self.source.to_kuzu_type(), self.source.to_kuzu_value()),
            (NodeProperty::MediaType, self.media_type.to_kuzu_type(), self.media_type.to_kuzu_value()),
            (NodeProperty::Select, self.select.to_kuzu_type(), self.select.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Target, self.target.to_kuzu_type(), self.target.to_kuzu_value()),
            (NodeProperty::Title, self.title.to_kuzu_type(), self.title.to_kuzu_value()),
            (NodeProperty::Rel, self.rel.to_kuzu_type(), self.rel.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Order, self.order.to_kuzu_type(), self.order.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::IsChecked, self.is_checked.to_kuzu_type(), self.is_checked.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::MathLanguage, self.math_language.to_kuzu_type(), self.math_language.to_kuzu_value()),
            (NodeProperty::Label, self.label.to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, self.label_automatically.to_kuzu_type(), self.label_automatically.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::MathLanguage, self.math_language.to_kuzu_type(), self.math_language.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.options.date_created.to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.options.date_received.to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.options.date_accepted.to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.options.date_modified.to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.options.date_published.to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Bitrate, self.options.bitrate.to_kuzu_type(), self.options.bitrate.to_kuzu_value()),
            (NodeProperty::ContentSize, self.options.content_size.to_kuzu_type(), self.options.content_size.to_kuzu_value()),
            (NodeProperty::ContentUrl, self.content_url.to_kuzu_type(), self.content_url.to_kuzu_value()),
            (NodeProperty::EmbedUrl, self.options.embed_url.to_kuzu_type(), self.options.embed_url.to_kuzu_value()),
            (NodeProperty::MediaType, self.media_type.to_kuzu_type(), self.media_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.options.references.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten()))
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::NoteType, self.note_type.to_kuzu_type(), self.note_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.ror.to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.name.to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Ror, self.ror.to_kuzu_type(), self.ror.to_kuzu_value()),
            (NodeProperty::LegalName, self.options.legal_name.to_kuzu_type(), self.options.legal_name.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Text, LogicalType::String, to_text(self).to_kuzu_value()),
            (embeddings_property(), embeddings_type(), Null.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::ExecutionMode, self.execution_mode.to_kuzu_type(), self.execution_mode.to_kuzu_value()),
            (NodeProperty::ExecutionCount, self.options.execution_count.to_kuzu_type(), self.options.execution_count.to_kuzu_value()),
            (NodeProperty::ExecutionRequired, self.options.execution_required.to_kuzu_type(), self.options.execution_required.to_kuzu_value()),
            (NodeProperty::ExecutionStatus, self.options.execution_status.to_kuzu_type(), self.options.execution_status.to_kuzu_value()),
            (NodeProperty::ExecutionEnded, self.options.execution_ended.to_kuzu_type(), self.options.execution_ended.to_kuzu_value()),
            (NodeProperty::ExecutionDuration, self.options.execution_duration.to_kuzu_type(), self.options.execution_duration.to_kuzu_value()),
            (NodeProperty::Name, self.name.to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Label, self.options.label.to_kuzu_type(), self.options.label.to_kuzu_value()),
            (NodeProperty::DerivedFrom, self.options.derived_from.to_kuzu_type(), self.options.derived_from.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.name.to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::DateStart, self.options.date_start.to_kuzu_type(), self.options.date_start.to_kuzu_value()),
            (NodeProperty::DateEnd, self.options.date_end.to_kuzu_type(), self.options.date_end.to_kuzu_value()),
            (NodeProperty::Issns, self.options.issns.to_kuzu_type(), self.options.issns.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            
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
    
    fn primary_key(&self) -> Value {
        self.orcid.to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Orcid, self.orcid.to_kuzu_type(), self.orcid.to_kuzu_value()),
            (NodeProperty::FamilyNames, self.family_names.to_kuzu_type(), self.family_names.to_kuzu_value()),
            (NodeProperty::GivenNames, self.given_names.to_kuzu_type(), self.given_names.to_kuzu_value()),
            (NodeProperty::HonorificPrefix, self.options.honorific_prefix.to_kuzu_type(), self.options.honorific_prefix.to_kuzu_value()),
            (NodeProperty::HonorificSuffix, self.options.honorific_suffix.to_kuzu_type(), self.options.honorific_suffix.to_kuzu_value()),
            (NodeProperty::Name, LogicalType::String, self.name().to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Affiliations, relations(self.affiliations.iter().flatten()))
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::IssueNumber, LogicalType::String, to_text(&self.issue_number).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::VolumeNumber, LogicalType::String, to_text(&self.volume_number).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Format, self.format.to_kuzu_type(), self.format.to_kuzu_value()),
            (NodeProperty::Content, self.content.to_kuzu_type(), self.content.to_kuzu_value()),
            (NodeProperty::Css, self.css.to_kuzu_type(), self.css.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Reference {
    fn node_type(&self) -> NodeType {
        NodeType::Reference
    }

    fn node_id(&self) -> NodeId {
        Reference::node_id(self)
    }
    
    fn primary_key(&self) -> Value {
        self.doi.to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::Date, self.date.to_kuzu_type(), self.date.to_kuzu_value()),
            (NodeProperty::Title, LogicalType::String, to_text(&self.title).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::SectionType, self.section_type.to_kuzu_type(), self.section_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten()))
        ]
    }
}

impl DatabaseNode for Sentence {
    fn node_type(&self) -> NodeType {
        NodeType::Sentence
    }

    fn node_id(&self) -> NodeId {
        Sentence::node_id(self)
    }
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Text, LogicalType::String, to_text(self).to_kuzu_value()),
            (embeddings_property(), embeddings_type(), Null.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Content, relations(self.content.iter()))
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.name.to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.options.date_created.to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.options.date_received.to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.options.date_accepted.to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.options.date_modified.to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.options.date_published.to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::SoftwareVersion, self.options.software_version.to_kuzu_type(), self.options.software_version.to_kuzu_value()),
            (NodeProperty::OperatingSystem, self.options.operating_system.to_kuzu_type(), self.options.operating_system.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.options.references.iter().flatten())),
            (NodeProperty::Title, relations(self.options.title.iter().flatten())),
            (NodeProperty::SoftwareRequirements, relations(self.options.software_requirements.iter().flatten()))
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::StyleLanguage, self.style_language.to_kuzu_type(), self.style_language.to_kuzu_value()),
            (NodeProperty::Css, self.options.css.to_kuzu_type(), self.options.css.to_kuzu_value()),
            (NodeProperty::ClassList, self.options.class_list.to_kuzu_type(), self.options.class_list.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Code, self.code.to_kuzu_type(), self.code.to_kuzu_value()),
            (NodeProperty::StyleLanguage, self.style_language.to_kuzu_type(), self.style_language.to_kuzu_value()),
            (NodeProperty::Css, self.options.css.to_kuzu_type(), self.options.css.to_kuzu_value()),
            (NodeProperty::ClassList, self.options.class_list.to_kuzu_type(), self.options.class_list.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.options.date_created.to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.options.date_received.to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.options.date_accepted.to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.options.date_modified.to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.options.date_published.to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Label, self.label.to_kuzu_type(), self.label.to_kuzu_value()),
            (NodeProperty::LabelAutomatically, self.label_automatically.to_kuzu_type(), self.label_automatically.to_kuzu_value()),
            (NodeProperty::Caption, LogicalType::String, to_text(&self.caption).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.options.references.iter().flatten())),
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::CellType, self.cell_type.to_kuzu_type(), self.cell_type.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::ColumnSpan, self.options.column_span.to_kuzu_type(), self.options.column_span.to_kuzu_value()),
            (NodeProperty::RowSpan, self.options.row_span.to_kuzu_type(), self.options.row_span.to_kuzu_value()),
            (NodeProperty::HorizontalAlignment, self.horizontal_alignment.to_kuzu_type(), self.horizontal_alignment.to_kuzu_value()),
            (NodeProperty::HorizontalAlignmentCharacter, self.horizontal_alignment_character.to_kuzu_type(), self.horizontal_alignment_character.to_kuzu_value()),
            (NodeProperty::VerticalAlignment, self.vertical_alignment.to_kuzu_type(), self.vertical_alignment.to_kuzu_value()),
            (NodeProperty::Text, LogicalType::String, to_text(self).to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::RowType, self.row_type.to_kuzu_type(), self.row_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::Name, self.name.to_kuzu_type(), self.name.to_kuzu_value()),
            (NodeProperty::ProgrammingLanguage, self.programming_language.to_kuzu_type(), self.programming_language.to_kuzu_value()),
            (NodeProperty::NativeType, self.native_type.to_kuzu_type(), self.native_type.to_kuzu_value()),
            (NodeProperty::NodeType, self.node_type.to_kuzu_type(), self.node_type.to_kuzu_value()),
            (NodeProperty::NativeHint, self.native_hint.to_kuzu_type(), self.native_hint.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
    
    fn primary_key(&self) -> Value {
        self.node_id().to_kuzu_value()
    }
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        vec![
            (NodeProperty::AlternateNames, self.options.alternate_names.to_kuzu_type(), self.options.alternate_names.to_kuzu_value()),
            (NodeProperty::Description, self.options.description.to_kuzu_type(), self.options.description.to_kuzu_value()),
            (NodeProperty::Name, self.options.name.to_kuzu_type(), self.options.name.to_kuzu_value()),
            (NodeProperty::Url, self.options.url.to_kuzu_type(), self.options.url.to_kuzu_value()),
            (NodeProperty::Doi, self.doi.to_kuzu_type(), self.doi.to_kuzu_value()),
            (NodeProperty::DateCreated, self.options.date_created.to_kuzu_type(), self.options.date_created.to_kuzu_value()),
            (NodeProperty::DateReceived, self.options.date_received.to_kuzu_type(), self.options.date_received.to_kuzu_value()),
            (NodeProperty::DateAccepted, self.options.date_accepted.to_kuzu_type(), self.options.date_accepted.to_kuzu_value()),
            (NodeProperty::DateModified, self.options.date_modified.to_kuzu_type(), self.options.date_modified.to_kuzu_value()),
            (NodeProperty::DatePublished, self.options.date_published.to_kuzu_type(), self.options.date_published.to_kuzu_value()),
            (NodeProperty::Genre, self.options.genre.to_kuzu_type(), self.options.genre.to_kuzu_value()),
            (NodeProperty::Keywords, self.options.keywords.to_kuzu_type(), self.options.keywords.to_kuzu_value()),
            (NodeProperty::Bitrate, self.options.bitrate.to_kuzu_type(), self.options.bitrate.to_kuzu_value()),
            (NodeProperty::ContentSize, self.options.content_size.to_kuzu_type(), self.options.content_size.to_kuzu_value()),
            (NodeProperty::ContentUrl, self.content_url.to_kuzu_type(), self.content_url.to_kuzu_value()),
            (NodeProperty::EmbedUrl, self.options.embed_url.to_kuzu_type(), self.options.embed_url.to_kuzu_value()),
            (NodeProperty::MediaType, self.media_type.to_kuzu_type(), self.media_type.to_kuzu_value())
        ]
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        vec![
            (NodeProperty::Abstract, relations(self.options.r#abstract.iter().flatten())),
            (NodeProperty::Authors, relations(self.options.authors.iter().flatten())),
            (NodeProperty::Contributors, relations(self.options.contributors.iter().flatten())),
            (NodeProperty::Editors, relations(self.options.editors.iter().flatten())),
            (NodeProperty::References, relations(self.options.references.iter().flatten())),
            (NodeProperty::Title, relations(self.title.iter().flatten())),
            (NodeProperty::Caption, relations(self.caption.iter().flatten()))
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
            Node::Citation(node) => node.node_type(),
            Node::CitationGroup(node) => node.node_type(),
            Node::Claim(node) => node.node_type(),
            Node::CodeBlock(node) => node.node_type(),
            Node::CodeChunk(node) => node.node_type(),
            Node::CodeExpression(node) => node.node_type(),
            Node::Directory(node) => node.node_type(),
            Node::Figure(node) => node.node_type(),
            Node::File(node) => node.node_type(),
            Node::ForBlock(node) => node.node_type(),
            Node::Function(node) => node.node_type(),
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
            Node::Note(node) => node.node_type(),
            Node::Organization(node) => node.node_type(),
            Node::Paragraph(node) => node.node_type(),
            Node::Parameter(node) => node.node_type(),
            Node::Periodical(node) => node.node_type(),
            Node::Person(node) => node.node_type(),
            Node::PublicationIssue(node) => node.node_type(),
            Node::PublicationVolume(node) => node.node_type(),
            Node::QuoteBlock(node) => node.node_type(),
            Node::QuoteInline(node) => node.node_type(),
            Node::RawBlock(node) => node.node_type(),
            Node::Reference(node) => node.node_type(),
            Node::Section(node) => node.node_type(),
            Node::Sentence(node) => node.node_type(),
            Node::SoftwareApplication(node) => node.node_type(),
            Node::StyledBlock(node) => node.node_type(),
            Node::StyledInline(node) => node.node_type(),
            Node::Table(node) => node.node_type(),
            Node::TableCell(node) => node.node_type(),
            Node::TableRow(node) => node.node_type(),
            Node::ThematicBreak(node) => node.node_type(),
            Node::Thing(node) => node.node_type(),
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
            Node::Citation(node) => node.node_id(),
            Node::CitationGroup(node) => node.node_id(),
            Node::Claim(node) => node.node_id(),
            Node::CodeBlock(node) => node.node_id(),
            Node::CodeChunk(node) => node.node_id(),
            Node::CodeExpression(node) => node.node_id(),
            Node::Directory(node) => node.node_id(),
            Node::Figure(node) => node.node_id(),
            Node::File(node) => node.node_id(),
            Node::ForBlock(node) => node.node_id(),
            Node::Function(node) => node.node_id(),
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
            Node::Note(node) => node.node_id(),
            Node::Organization(node) => node.node_id(),
            Node::Paragraph(node) => node.node_id(),
            Node::Parameter(node) => node.node_id(),
            Node::Periodical(node) => node.node_id(),
            Node::Person(node) => node.node_id(),
            Node::PublicationIssue(node) => node.node_id(),
            Node::PublicationVolume(node) => node.node_id(),
            Node::QuoteBlock(node) => node.node_id(),
            Node::QuoteInline(node) => node.node_id(),
            Node::RawBlock(node) => node.node_id(),
            Node::Reference(node) => node.node_id(),
            Node::Section(node) => node.node_id(),
            Node::Sentence(node) => node.node_id(),
            Node::SoftwareApplication(node) => node.node_id(),
            Node::StyledBlock(node) => node.node_id(),
            Node::StyledInline(node) => node.node_id(),
            Node::Table(node) => node.node_id(),
            Node::TableCell(node) => node.node_id(),
            Node::TableRow(node) => node.node_id(),
            Node::ThematicBreak(node) => node.node_id(),
            Node::Thing(node) => node.node_id(),
            Node::Variable(node) => node.node_id(),
            Node::VideoObject(node) => node.node_id(),
            _ => NodeId::null()
        }
    }

    fn primary_key(&self) -> Value {
        match self {
            Node::Admonition(node) => node.primary_key(),
            Node::Annotation(node) => node.primary_key(),
            Node::Article(node) => node.primary_key(),
            Node::AudioObject(node) => node.primary_key(),
            Node::AuthorRole(node) => node.primary_key(),
            Node::Citation(node) => node.primary_key(),
            Node::CitationGroup(node) => node.primary_key(),
            Node::Claim(node) => node.primary_key(),
            Node::CodeBlock(node) => node.primary_key(),
            Node::CodeChunk(node) => node.primary_key(),
            Node::CodeExpression(node) => node.primary_key(),
            Node::Directory(node) => node.primary_key(),
            Node::Figure(node) => node.primary_key(),
            Node::File(node) => node.primary_key(),
            Node::ForBlock(node) => node.primary_key(),
            Node::Function(node) => node.primary_key(),
            Node::Heading(node) => node.primary_key(),
            Node::IfBlock(node) => node.primary_key(),
            Node::IfBlockClause(node) => node.primary_key(),
            Node::ImageObject(node) => node.primary_key(),
            Node::IncludeBlock(node) => node.primary_key(),
            Node::Link(node) => node.primary_key(),
            Node::List(node) => node.primary_key(),
            Node::ListItem(node) => node.primary_key(),
            Node::MathBlock(node) => node.primary_key(),
            Node::MathInline(node) => node.primary_key(),
            Node::MediaObject(node) => node.primary_key(),
            Node::Note(node) => node.primary_key(),
            Node::Organization(node) => node.primary_key(),
            Node::Paragraph(node) => node.primary_key(),
            Node::Parameter(node) => node.primary_key(),
            Node::Periodical(node) => node.primary_key(),
            Node::Person(node) => node.primary_key(),
            Node::PublicationIssue(node) => node.primary_key(),
            Node::PublicationVolume(node) => node.primary_key(),
            Node::QuoteBlock(node) => node.primary_key(),
            Node::QuoteInline(node) => node.primary_key(),
            Node::RawBlock(node) => node.primary_key(),
            Node::Reference(node) => node.primary_key(),
            Node::Section(node) => node.primary_key(),
            Node::Sentence(node) => node.primary_key(),
            Node::SoftwareApplication(node) => node.primary_key(),
            Node::StyledBlock(node) => node.primary_key(),
            Node::StyledInline(node) => node.primary_key(),
            Node::Table(node) => node.primary_key(),
            Node::TableCell(node) => node.primary_key(),
            Node::TableRow(node) => node.primary_key(),
            Node::ThematicBreak(node) => node.primary_key(),
            Node::Thing(node) => node.primary_key(),
            Node::Variable(node) => node.primary_key(),
            Node::VideoObject(node) => node.primary_key(),
            _ => Value::Null(LogicalType::Any)
        }
    }

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        match self {
            Node::Admonition(node) => node.node_table(),
            Node::Annotation(node) => node.node_table(),
            Node::Article(node) => node.node_table(),
            Node::AudioObject(node) => node.node_table(),
            Node::AuthorRole(node) => node.node_table(),
            Node::Citation(node) => node.node_table(),
            Node::CitationGroup(node) => node.node_table(),
            Node::Claim(node) => node.node_table(),
            Node::CodeBlock(node) => node.node_table(),
            Node::CodeChunk(node) => node.node_table(),
            Node::CodeExpression(node) => node.node_table(),
            Node::Directory(node) => node.node_table(),
            Node::Figure(node) => node.node_table(),
            Node::File(node) => node.node_table(),
            Node::ForBlock(node) => node.node_table(),
            Node::Function(node) => node.node_table(),
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
            Node::Note(node) => node.node_table(),
            Node::Organization(node) => node.node_table(),
            Node::Paragraph(node) => node.node_table(),
            Node::Parameter(node) => node.node_table(),
            Node::Periodical(node) => node.node_table(),
            Node::Person(node) => node.node_table(),
            Node::PublicationIssue(node) => node.node_table(),
            Node::PublicationVolume(node) => node.node_table(),
            Node::QuoteBlock(node) => node.node_table(),
            Node::QuoteInline(node) => node.node_table(),
            Node::RawBlock(node) => node.node_table(),
            Node::Reference(node) => node.node_table(),
            Node::Section(node) => node.node_table(),
            Node::Sentence(node) => node.node_table(),
            Node::SoftwareApplication(node) => node.node_table(),
            Node::StyledBlock(node) => node.node_table(),
            Node::StyledInline(node) => node.node_table(),
            Node::Table(node) => node.node_table(),
            Node::TableCell(node) => node.node_table(),
            Node::TableRow(node) => node.node_table(),
            Node::ThematicBreak(node) => node.node_table(),
            Node::Thing(node) => node.node_table(),
            Node::Variable(node) => node.node_table(),
            Node::VideoObject(node) => node.node_table(),
            _ => Vec::new()
        }
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        match self {
            Node::Admonition(node) => node.rel_tables(),
            Node::Annotation(node) => node.rel_tables(),
            Node::Article(node) => node.rel_tables(),
            Node::AudioObject(node) => node.rel_tables(),
            Node::AuthorRole(node) => node.rel_tables(),
            Node::Citation(node) => node.rel_tables(),
            Node::CitationGroup(node) => node.rel_tables(),
            Node::Claim(node) => node.rel_tables(),
            Node::CodeBlock(node) => node.rel_tables(),
            Node::CodeChunk(node) => node.rel_tables(),
            Node::CodeExpression(node) => node.rel_tables(),
            Node::Directory(node) => node.rel_tables(),
            Node::Figure(node) => node.rel_tables(),
            Node::File(node) => node.rel_tables(),
            Node::ForBlock(node) => node.rel_tables(),
            Node::Function(node) => node.rel_tables(),
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
            Node::Note(node) => node.rel_tables(),
            Node::Organization(node) => node.rel_tables(),
            Node::Paragraph(node) => node.rel_tables(),
            Node::Parameter(node) => node.rel_tables(),
            Node::Periodical(node) => node.rel_tables(),
            Node::Person(node) => node.rel_tables(),
            Node::PublicationIssue(node) => node.rel_tables(),
            Node::PublicationVolume(node) => node.rel_tables(),
            Node::QuoteBlock(node) => node.rel_tables(),
            Node::QuoteInline(node) => node.rel_tables(),
            Node::RawBlock(node) => node.rel_tables(),
            Node::Reference(node) => node.rel_tables(),
            Node::Section(node) => node.rel_tables(),
            Node::Sentence(node) => node.rel_tables(),
            Node::SoftwareApplication(node) => node.rel_tables(),
            Node::StyledBlock(node) => node.rel_tables(),
            Node::StyledInline(node) => node.rel_tables(),
            Node::Table(node) => node.rel_tables(),
            Node::TableCell(node) => node.rel_tables(),
            Node::TableRow(node) => node.rel_tables(),
            Node::ThematicBreak(node) => node.rel_tables(),
            Node::Thing(node) => node.rel_tables(),
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

    fn primary_key(&self) -> Value {
        match self {
            Block::Admonition(node) => node.primary_key(),
            Block::AudioObject(node) => node.primary_key(),
            Block::Claim(node) => node.primary_key(),
            Block::CodeBlock(node) => node.primary_key(),
            Block::CodeChunk(node) => node.primary_key(),
            Block::Figure(node) => node.primary_key(),
            Block::File(node) => node.primary_key(),
            Block::ForBlock(node) => node.primary_key(),
            Block::Heading(node) => node.primary_key(),
            Block::IfBlock(node) => node.primary_key(),
            Block::ImageObject(node) => node.primary_key(),
            Block::IncludeBlock(node) => node.primary_key(),
            Block::List(node) => node.primary_key(),
            Block::MathBlock(node) => node.primary_key(),
            Block::Paragraph(node) => node.primary_key(),
            Block::QuoteBlock(node) => node.primary_key(),
            Block::RawBlock(node) => node.primary_key(),
            Block::Section(node) => node.primary_key(),
            Block::StyledBlock(node) => node.primary_key(),
            Block::Table(node) => node.primary_key(),
            Block::ThematicBreak(node) => node.primary_key(),
            Block::VideoObject(node) => node.primary_key(),
            _ => Value::Null(LogicalType::Any)
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
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
            Inline::Citation(node) => node.node_type(),
            Inline::CitationGroup(node) => node.node_type(),
            Inline::CodeExpression(node) => node.node_type(),
            Inline::ImageObject(node) => node.node_type(),
            Inline::Link(node) => node.node_type(),
            Inline::MathInline(node) => node.node_type(),
            Inline::MediaObject(node) => node.node_type(),
            Inline::Note(node) => node.node_type(),
            Inline::Parameter(node) => node.node_type(),
            Inline::QuoteInline(node) => node.node_type(),
            Inline::Sentence(node) => node.node_type(),
            Inline::StyledInline(node) => node.node_type(),
            Inline::VideoObject(node) => node.node_type(),
            _ => NodeType::Unknown
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Inline::Annotation(node) => node.node_id(),
            Inline::AudioObject(node) => node.node_id(),
            Inline::Citation(node) => node.node_id(),
            Inline::CitationGroup(node) => node.node_id(),
            Inline::CodeExpression(node) => node.node_id(),
            Inline::ImageObject(node) => node.node_id(),
            Inline::Link(node) => node.node_id(),
            Inline::MathInline(node) => node.node_id(),
            Inline::MediaObject(node) => node.node_id(),
            Inline::Note(node) => node.node_id(),
            Inline::Parameter(node) => node.node_id(),
            Inline::QuoteInline(node) => node.node_id(),
            Inline::Sentence(node) => node.node_id(),
            Inline::StyledInline(node) => node.node_id(),
            Inline::VideoObject(node) => node.node_id(),
            _ => NodeId::null()
        }
    }

    fn primary_key(&self) -> Value {
        match self {
            Inline::Annotation(node) => node.primary_key(),
            Inline::AudioObject(node) => node.primary_key(),
            Inline::Citation(node) => node.primary_key(),
            Inline::CitationGroup(node) => node.primary_key(),
            Inline::CodeExpression(node) => node.primary_key(),
            Inline::ImageObject(node) => node.primary_key(),
            Inline::Link(node) => node.primary_key(),
            Inline::MathInline(node) => node.primary_key(),
            Inline::MediaObject(node) => node.primary_key(),
            Inline::Note(node) => node.primary_key(),
            Inline::Parameter(node) => node.primary_key(),
            Inline::QuoteInline(node) => node.primary_key(),
            Inline::Sentence(node) => node.primary_key(),
            Inline::StyledInline(node) => node.primary_key(),
            Inline::VideoObject(node) => node.primary_key(),
            _ => Value::Null(LogicalType::Any)
        }
    }

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        match self {
            Inline::Annotation(node) => node.node_table(),
            Inline::AudioObject(node) => node.node_table(),
            Inline::Citation(node) => node.node_table(),
            Inline::CitationGroup(node) => node.node_table(),
            Inline::CodeExpression(node) => node.node_table(),
            Inline::ImageObject(node) => node.node_table(),
            Inline::Link(node) => node.node_table(),
            Inline::MathInline(node) => node.node_table(),
            Inline::MediaObject(node) => node.node_table(),
            Inline::Note(node) => node.node_table(),
            Inline::Parameter(node) => node.node_table(),
            Inline::QuoteInline(node) => node.node_table(),
            Inline::Sentence(node) => node.node_table(),
            Inline::StyledInline(node) => node.node_table(),
            Inline::VideoObject(node) => node.node_table(),
            _ => Vec::new()
        }
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        match self {
            Inline::Annotation(node) => node.rel_tables(),
            Inline::AudioObject(node) => node.rel_tables(),
            Inline::Citation(node) => node.rel_tables(),
            Inline::CitationGroup(node) => node.rel_tables(),
            Inline::CodeExpression(node) => node.rel_tables(),
            Inline::ImageObject(node) => node.rel_tables(),
            Inline::Link(node) => node.rel_tables(),
            Inline::MathInline(node) => node.rel_tables(),
            Inline::MediaObject(node) => node.rel_tables(),
            Inline::Note(node) => node.rel_tables(),
            Inline::Parameter(node) => node.rel_tables(),
            Inline::QuoteInline(node) => node.rel_tables(),
            Inline::Sentence(node) => node.rel_tables(),
            Inline::StyledInline(node) => node.rel_tables(),
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

    fn primary_key(&self) -> Value {
        match self {
            Author::Person(node) => node.primary_key(),
            Author::Organization(node) => node.primary_key(),
            Author::SoftwareApplication(node) => node.primary_key(),
            Author::AuthorRole(node) => node.primary_key(),
            _ => Value::Null(LogicalType::Any)
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

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        match self {
            Author::Person(node) => node.rel_tables(),
            Author::Organization(node) => node.rel_tables(),
            Author::SoftwareApplication(node) => node.rel_tables(),
            Author::AuthorRole(node) => node.rel_tables(),
            _ => Vec::new()
        }
    }
}

#[allow(unreachable_patterns)]
impl DatabaseNode for AuthorRoleAuthor {
    fn node_type(&self) -> NodeType {
        match self {
            AuthorRoleAuthor::Person(node) => node.node_type(),
            AuthorRoleAuthor::Organization(node) => node.node_type(),
            AuthorRoleAuthor::SoftwareApplication(node) => node.node_type(),
            AuthorRoleAuthor::Thing(node) => node.node_type(),
            _ => NodeType::Unknown
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            AuthorRoleAuthor::Person(node) => node.node_id(),
            AuthorRoleAuthor::Organization(node) => node.node_id(),
            AuthorRoleAuthor::SoftwareApplication(node) => node.node_id(),
            AuthorRoleAuthor::Thing(node) => node.node_id(),
            _ => NodeId::null()
        }
    }

    fn primary_key(&self) -> Value {
        match self {
            AuthorRoleAuthor::Person(node) => node.primary_key(),
            AuthorRoleAuthor::Organization(node) => node.primary_key(),
            AuthorRoleAuthor::SoftwareApplication(node) => node.primary_key(),
            AuthorRoleAuthor::Thing(node) => node.primary_key(),
            _ => Value::Null(LogicalType::Any)
        }
    }

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {
        match self {
            AuthorRoleAuthor::Person(node) => node.node_table(),
            AuthorRoleAuthor::Organization(node) => node.node_table(),
            AuthorRoleAuthor::SoftwareApplication(node) => node.node_table(),
            AuthorRoleAuthor::Thing(node) => node.node_table(),
            _ => Vec::new()
        }
    }

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {
        match self {
            AuthorRoleAuthor::Person(node) => node.rel_tables(),
            AuthorRoleAuthor::Organization(node) => node.rel_tables(),
            AuthorRoleAuthor::SoftwareApplication(node) => node.rel_tables(),
            AuthorRoleAuthor::Thing(node) => node.rel_tables(),
            _ => Vec::new()
        }
    }
}

