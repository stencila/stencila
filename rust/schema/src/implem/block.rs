use crate::{prelude::*, Block, Inline, Node, Paragraph, Section};

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
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            DeleteBlock,
            Figure,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            IncludeBlock,
            InsertBlock,
            InstructionBlock,
            List,
            MathBlock,
            ModifyBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            ReplaceBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Table,
            ThematicBreak,
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
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            DeleteBlock,
            Figure,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            IncludeBlock,
            InsertBlock,
            InstructionBlock,
            List,
            MathBlock,
            ModifyBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            ReplaceBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Table,
            ThematicBreak,
            Walkthrough
        )
    }
}

impl TryFrom<Node> for Block {
    type Error = ErrReport;

    fn try_from(node: Node) -> Result<Self> {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match node {
                    $(Node::$variant(node) => Ok(Block::$variant(node)),)*
                    _ => bail!("Unable to convert node to block")
                }
            };
        }

        variants!(
            Admonition,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            DeleteBlock,
            Figure,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            IncludeBlock,
            InsertBlock,
            InstructionBlock,
            List,
            MathBlock,
            ModifyBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            ReplaceBlock,
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
            Figure(block) => block.content,
            ForBlock(block) => block.content,
            QuoteBlock(block) => block.content,
            Section(block) => block.content,
            StyledBlock(block) => block.content,
            SuggestionBlock(block) => block.content,

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
            // These block types are never encoded to LLMd
            if matches!(
                self,
                Block::SuggestionBlock(..)
                    | Block::InsertBlock(..)
                    | Block::DeleteBlock(..)
                    | Block::ModifyBlock(..)
                    | Block::ReplaceBlock(..)
            ) {
                return;
            }

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
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            DeleteBlock,
            Figure,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            IncludeBlock,
            InsertBlock,
            InstructionBlock,
            List,
            MathBlock,
            ModifyBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            ReplaceBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Table,
            ThematicBreak,
            Walkthrough
        )
    }
}
