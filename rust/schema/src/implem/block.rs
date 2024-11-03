use crate::{prelude::*, Block, Inline, Paragraph, Section};

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
