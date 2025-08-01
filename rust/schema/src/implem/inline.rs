use node_store::{ReadNode, ReadStore, automerge::ObjId, get_node_type};

use crate::{prelude::*, transforms::blocks_to_inlines, *};

impl Inline {
    pub fn node_type(&self) -> NodeType {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match self {
                    $(Inline::$variant(node) => node.node_type(),)*

                    Inline::Null(..) => NodeType::Null,
                    Inline::Boolean(..) => NodeType::Boolean,
                    Inline::Integer(..) => NodeType::Integer,
                    Inline::UnsignedInteger(..) => NodeType::UnsignedInteger,
                    Inline::Number(..) => NodeType::Number,
                }
            };
        }

        variants!(
            Annotation,
            AudioObject,
            Button,
            Citation,
            CitationGroup,
            CodeExpression,
            CodeInline,
            Date,
            DateTime,
            Duration,
            Emphasis,
            ImageObject,
            InstructionInline,
            Link,
            MathInline,
            MediaObject,
            Note,
            Parameter,
            QuoteInline,
            Sentence,
            Strikeout,
            Strong,
            StyledInline,
            Subscript,
            SuggestionInline,
            Superscript,
            Text,
            Time,
            Timestamp,
            Underline,
            VideoObject
        )
    }

    pub fn node_id(&self) -> Option<NodeId> {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match self {
                    $(Inline::$variant(node) => Some(node.node_id()),)*

                    Inline::Null(..) |
                    Inline::Boolean(..) |
                    Inline::Integer(..) |
                    Inline::UnsignedInteger(..) |
                    Inline::Number(..) => None,
                }
            };
        }

        variants!(
            Annotation,
            AudioObject,
            Button,
            Citation,
            CitationGroup,
            CodeExpression,
            CodeInline,
            Date,
            DateTime,
            Duration,
            Emphasis,
            ImageObject,
            InstructionInline,
            Link,
            MathInline,
            MediaObject,
            Note,
            Parameter,
            QuoteInline,
            Sentence,
            Strikeout,
            Strong,
            StyledInline,
            Subscript,
            SuggestionInline,
            Superscript,
            Text,
            Time,
            Timestamp,
            Underline,
            VideoObject
        )
    }
}

impl ReadNode for Inline {
    fn load_null() -> Result<Self> {
        Ok(Inline::Null(Null {}))
    }

    fn load_boolean(value: &bool) -> Result<Self> {
        Ok(Inline::Boolean(*value))
    }

    fn load_int(value: &i64) -> Result<Self> {
        Ok(Inline::Integer(*value))
    }

    fn load_uint(value: &u64) -> Result<Self> {
        Ok(Inline::UnsignedInteger(*value))
    }

    fn load_f64(value: &f64) -> Result<Self> {
        Ok(Inline::Number(*value))
    }

    fn load_map<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        let Some(node_type) = get_node_type(store, obj_id)? else {
            bail!("Object in Automerge store is not an `Inline`");
        };

        macro_rules! load_map_variants {
            ($( $variant:ident ),*) => {
                match node_type {
                    $(
                        NodeType::$variant => Ok(Inline::$variant(crate::$variant::load_map(store, obj_id)?)),
                    )*

                    _ => bail!("Unexpected type `{node_type}` in Automerge store for `Inline`"),
                }
            };
        }

        load_map_variants!(
            AudioObject,
            Button,
            Citation,
            CitationGroup,
            CodeExpression,
            CodeInline,
            Date,
            DateTime,
            Duration,
            Emphasis,
            ImageObject,
            InstructionInline,
            Link,
            MathInline,
            MediaObject,
            Note,
            Parameter,
            QuoteInline,
            Strikeout,
            Strong,
            StyledInline,
            Subscript,
            Superscript,
            Text,
            Time,
            Timestamp,
            Underline,
            VideoObject
        )
    }
}

impl From<Vec<Inline>> for Inline {
    fn from(mut inlines: Vec<Inline>) -> Self {
        if inlines.len() == 1 {
            // Take first inline
            inlines.swap_remove(0)
        } else {
            // Collapse inlines into a single inline text node
            Inline::Text(Text::from(inlines.to_text().0))
        }
    }
}

impl From<Block> for Inline {
    fn from(block: Block) -> Self {
        match block {
            // Blocks that can also be inlines
            Block::AudioObject(obj) => Inline::AudioObject(obj),
            Block::ImageObject(obj) => Inline::ImageObject(obj),
            Block::VideoObject(obj) => Inline::VideoObject(obj),

            // Blocks with inline analogues
            Block::CodeBlock(code_block) => Inline::CodeInline(CodeInline {
                code: code_block.code,
                programming_language: code_block.programming_language,
                ..Default::default()
            }),
            Block::MathBlock(math_block) => Inline::MathInline(MathInline {
                code: math_block.code,
                math_language: math_block.math_language,
                ..Default::default()
            }),
            Block::QuoteBlock(quote_block) => Inline::QuoteInline(QuoteInline {
                content: blocks_to_inlines(quote_block.content),
                source: quote_block.source,
                ..Default::default()
            }),

            // Blocks with inline content
            Block::Heading(heading) => heading.content.into(),
            Block::Paragraph(paragraph) => paragraph.content.into(),

            // Blocks with block content
            Block::Claim(claim) => claim.content.into(),
            Block::IncludeBlock(IncludeBlock {
                source, content, ..
            })
            | Block::CallBlock(CallBlock {
                source, content, ..
            }) => match content {
                Some(content) => content.into(),
                None => Inline::Text(Text::from(source)),
            },

            // Fallback to inline text
            _ => Inline::Text(Text::from(block.to_text().0)),
        }
    }
}

impl From<Vec<Block>> for Inline {
    fn from(mut blocks: Vec<Block>) -> Self {
        if blocks.len() == 1 {
            // Transform first block to inlines
            blocks.swap_remove(0).into()
        } else {
            // Transform blocks to inlines and wrap in an inline span
            Inline::StyledInline(StyledInline {
                content: blocks_to_inlines(blocks),
                ..Default::default()
            })
        }
    }
}

impl From<Block> for Vec<Inline> {
    fn from(block: Block) -> Self {
        match &block {
            // Variants with inline content
            Block::Heading(heading) => heading.content.to_owned(),
            Block::Paragraph(paragraph) => paragraph.content.to_owned(),

            // Variants with block content
            Block::Claim(claim) => blocks_to_inlines(claim.content.to_owned()),
            Block::IncludeBlock(IncludeBlock { content, .. })
            | Block::CallBlock(CallBlock { content, .. }) => match &content {
                Some(content) => blocks_to_inlines(content.to_owned()),
                None => vec![block.into()],
            },

            // Fallback to a single item vector of `block` transformed to an inline
            _ => vec![block.into()],
        }
    }
}

impl TryFrom<Node> for Inline {
    type Error = ErrReport;

    fn try_from(node: Node) -> Result<Self> {
        // Inlines are directly convertible
        macro_rules! inlines {
            ($( $variant:ident ),*) => {
                match node {
                    $(Node::$variant(node) => Ok(Inline::$variant(node)),)*
                    _ => bail!("Unable to convert Node::{} to Inline", node.node_type())
                }
            };
        }
        inlines!(
            Annotation,
            AudioObject,
            Button,
            Citation,
            CitationGroup,
            CodeExpression,
            CodeInline,
            Date,
            DateTime,
            Duration,
            Emphasis,
            ImageObject,
            InstructionInline,
            Link,
            MathInline,
            MediaObject,
            Note,
            Parameter,
            QuoteInline,
            Sentence,
            Strikeout,
            Strong,
            StyledInline,
            Subscript,
            SuggestionInline,
            Superscript,
            Text,
            Time,
            Timestamp,
            Underline,
            VideoObject
        )
    }
}
