use crate::{prelude::*, Block, Inline, List, Node, Paragraph, Section, Table, TableRow};

impl Block {
    pub fn node_type(&self) -> NodeType {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match self {
                    $(Block::$variant(node) => node.node_type(),)*
                }
            };
        }

        variants!(
            Admonition,
            AudioObject,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            Excerpt,
            Figure,
            File,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            ImageObject,
            IncludeBlock,
            InstructionBlock,
            List,
            MathBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Table,
            ThematicBreak,
            VideoObject,
            Walkthrough
        )
    }

    pub fn node_id(&self) -> Option<NodeId> {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match self {
                    $(Block::$variant(node) => Some(node.node_id()),)*
                }
            };
        }

        variants!(
            Admonition,
            AudioObject,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            Excerpt,
            Figure,
            File,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            ImageObject,
            IncludeBlock,
            InstructionBlock,
            List,
            MathBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Table,
            ThematicBreak,
            VideoObject,
            Walkthrough
        )
    }
}

impl TryFrom<Node> for Block {
    type Error = ErrReport;

    fn try_from(node: Node) -> Result<Self> {
        // Wrap parts of blocks (e.g. table cells, table rows, list items) accordingly
        match node {
            Node::TableCell(cell) => {
                return Ok(Block::Table(Table::new(vec![TableRow::new(vec![cell])])))
            }
            Node::TableRow(row) => return Ok(Block::Table(Table::new(vec![row]))),
            Node::ListItem(item) => {
                return Ok(Block::List(List::new(
                    vec![item],
                    crate::ListOrder::Unordered,
                )))
            }
            _ => {}
        }

        // Inlines are wrapped in a paragraph
        macro_rules! inlines {
            ($( $variant:ident ),*) => {
                match node {
                    $(Node::$variant(node) => return Ok(Block::Paragraph(Paragraph::new(vec![Inline::$variant(node)]))),)*
                    _ => {}
                }
            };
        }
        inlines!(
            Annotation,
            AudioObject,
            Button,
            Cite,
            CiteGroup,
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
            SuggestionInline,
            Superscript,
            Text,
            Time,
            Timestamp,
            Underline,
            VideoObject,
            // Primitive inlines also wrapped into paragraphs
            Boolean,
            Integer,
            UnsignedInteger,
            Number
        );

        // Blocks are directly convertible
        macro_rules! blocks {
            ($( $variant:ident ),*) => {
                match node {
                    $(Node::$variant(node) => Ok(Block::$variant(node)),)*
                    _ => bail!("Unable to convert node to block")
                }
            };
        }
        blocks!(
            Admonition,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            Excerpt,
            Figure,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            IncludeBlock,
            InstructionBlock,
            List,
            MathBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Table,
            ThematicBreak,
            Walkthrough
        )
    }
}

impl TryFrom<Node> for Vec<Block> {
    type Error = ErrReport;

    fn try_from(node: Node) -> Result<Self> {
        use Node::*;
        Ok(match node {
            // For creative works with block content, return that content
            Article(node) => node.content,
            Chat(node) => node.content,
            Prompt(node) => node.content,

            // For block nodes with block content, return that content
            Admonition(block) => block.content,
            CallBlock(block) => block.content.unwrap_or_default(),
            ChatMessage(block) => block.content,
            Claim(block) => block.content,
            Excerpt(block) => block.content,
            Figure(block) => block.content,
            ForBlock(block) => block.content,
            QuoteBlock(block) => block.content,
            Section(block) => block.content,
            StyledBlock(block) => block.content,
            SuggestionBlock(block) => block.content,
            TableCell(block) => block.content,

            // For other node types, attempt to return a vector with a single block
            _ => vec![Block::try_from(node)?],
        })
    }
}

impl From<Vec<Block>> for Block {
    fn from(blocks: Vec<Block>) -> Block {
        Block::Section(Section {
            content: blocks,
            ..Default::default()
        })
    }
}

impl From<Vec<Inline>> for Block {
    fn from(inlines: Vec<Inline>) -> Block {
        Block::Paragraph(Paragraph {
            content: inlines,
            ..Default::default()
        })
    }
}

impl MarkdownCodec for Block {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(context.format, Format::Llmd) {
            // Most other block types are only encoded to LLMd if they meet the specified
            // thresholds for provenance. This is not done for compound block types (those that
            // have nested blocks) where the provenance is calculated for the 'code' of the
            // block (e.g. an instruction could be machine written but its content verified).
            if !matches!(
                self,
                Block::CallBlock(..)
                    | Block::ForBlock(..)
                    | Block::IfBlock(..)
                    | Block::IncludeBlock(..)
                    | Block::InstructionBlock(..)
                    | Block::StyledBlock(..)
            ) {
                if let Some(provenance) = self.provenance() {
                    let human = ProvenanceCount::human_percent(&provenance);
                    let verified = ProvenanceCount::verified_percent(&provenance);
                    if human < 50 && verified < 50 {
                        return;
                    }
                }
            }
        }

        // Default handling for other Markdown formats and if for
        // LLMd if provenance criteria are met (some of these have specific handling for LLMd
        // that is usually similar/same to handling for `context.render`)
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match self {
                    $(Block::$variant(node) => node.to_markdown(context),)*
                }
            };
        }
        variants!(
            Admonition,
            AudioObject,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            Excerpt,
            Figure,
            File,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            ImageObject,
            IncludeBlock,
            InstructionBlock,
            List,
            MathBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Table,
            ThematicBreak,
            VideoObject,
            Walkthrough
        )
    }
}
