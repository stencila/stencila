use stencila_node_id::NodeUid;

use crate::{Block, Inline, Node, NodeType, Object, ResearchObjectRelation};

/// A borrowed view of the fields shared by every concrete research object.
#[derive(Debug, Clone, Copy)]
pub struct ResearchObjectRef<'a> {
    kind: NodeType,
    id: Option<&'a str>,
    label: Option<&'a str>,
    title: Option<&'a [Inline]>,
    content: &'a [Block],
    relations: Option<&'a [ResearchObjectRelation]>,
    extra: Option<&'a Object>,
}

impl<'a> ResearchObjectRef<'a> {
    /// The concrete Schema type of the research object.
    pub fn kind(self) -> NodeType {
        self.kind
    }

    /// The authored identifier, if present.
    pub fn id(self) -> Option<&'a str> {
        self.id
    }

    /// The authored label, if present.
    pub fn label(self) -> Option<&'a str> {
        self.label
    }

    /// The authored title, if present.
    pub fn title(self) -> Option<&'a [Inline]> {
        self.title
    }

    /// The block content of the research object.
    pub fn content(self) -> &'a [Block] {
        self.content
    }

    /// The structured outbound research relations.
    pub fn relations(self) -> Option<&'a [ResearchObjectRelation]> {
        self.relations
    }

    /// Interoperability metadata not represented by core Schema properties.
    pub fn extra(self) -> Option<&'a Object> {
        self.extra
    }
}

/// A mutable view of fields shared by every concrete research object.
pub struct ResearchObjectMut<'a> {
    id: Option<&'a str>,
    title: &'a mut Option<Vec<Inline>>,
    content: &'a mut Vec<Block>,
    relations: &'a mut Option<Vec<ResearchObjectRelation>>,
    extra: &'a mut Option<Object>,
    uid: &'a mut NodeUid,
}

/// A mutable view of the common authored research-object fields.
pub struct ResearchObjectFieldsMut<'a> {
    /// The optional authored title.
    pub title: &'a mut Option<Vec<Inline>>,
    /// The body blocks.
    pub content: &'a mut Vec<Block>,
    /// Structured research relations.
    pub relations: &'a mut Option<Vec<ResearchObjectRelation>>,
    /// Additional interoperability metadata.
    pub extra: &'a mut Option<Object>,
}

impl<'a> ResearchObjectMut<'a> {
    /// The authored identifier, if present.
    pub fn id(&self) -> Option<&str> {
        self.id
    }

    /// Set the internal UID used to construct stable node identifiers.
    pub fn set_uid(&mut self, uid: NodeUid) {
        *self.uid = uid;
    }

    /// Borrow the common authored fields.
    pub fn fields(&mut self) -> ResearchObjectFieldsMut<'_> {
        ResearchObjectFieldsMut {
            title: self.title,
            content: self.content,
            relations: self.relations,
            extra: self.extra,
        }
    }
}

macro_rules! research_object_ref {
    ($node:ident, $kind:ident) => {
        ResearchObjectRef {
            kind: NodeType::$kind,
            id: $node.id.as_deref(),
            label: $node.label.as_deref(),
            title: $node.options.title.as_deref(),
            content: &$node.content,
            relations: $node.relations.as_deref(),
            extra: $node.options.extra.as_ref(),
        }
    };
}

macro_rules! research_object_mut {
    ($node:ident) => {
        ResearchObjectMut {
            id: $node.id.as_deref(),
            title: &mut $node.options.title,
            content: &mut $node.content,
            relations: &mut $node.relations,
            extra: &mut $node.options.extra,
            uid: &mut $node.uid,
        }
    };
}

macro_rules! impl_research_object_access {
    ($enum:ident) => {
        impl $enum {
            /// Borrow this node as a research object when it is one.
            pub fn as_research_object(&self) -> Option<ResearchObjectRef<'_>> {
                Some(match self {
                    Self::Claim(node) => research_object_ref!(node, Claim),
                    Self::Evidence(node) => research_object_ref!(node, Evidence),
                    Self::Protocol(node) => research_object_ref!(node, Protocol),
                    Self::Question(node) => research_object_ref!(node, Question),
                    Self::Request(node) => research_object_ref!(node, Request),
                    _ => return None,
                })
            }

            /// Mutably borrow this node as a research object when it is one.
            pub fn as_research_object_mut(&mut self) -> Option<ResearchObjectMut<'_>> {
                Some(match self {
                    Self::Claim(node) => research_object_mut!(node),
                    Self::Evidence(node) => research_object_mut!(node),
                    Self::Protocol(node) => research_object_mut!(node),
                    Self::Question(node) => research_object_mut!(node),
                    Self::Request(node) => research_object_mut!(node),
                    _ => return None,
                })
            }
        }
    };
}

impl_research_object_access!(Node);
impl_research_object_access!(Block);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Claim, Evidence};
    use eyre::Result;

    #[test]
    fn views_node_and_block_research_objects() -> Result<()> {
        let mut claim = Claim::new(Vec::new());
        claim.id = Some("claim-1".to_string());
        let node = Node::Claim(claim);
        let view = node
            .as_research_object()
            .ok_or_else(|| eyre::eyre!("claim should be a research object"))?;
        assert_eq!(view.kind(), NodeType::Claim);
        assert_eq!(view.id(), Some("claim-1"));

        let block = Block::Evidence(Evidence::new(Vec::new()));
        assert_eq!(
            block.as_research_object().map(ResearchObjectRef::kind),
            Some(NodeType::Evidence)
        );
        Ok(())
    }
}
