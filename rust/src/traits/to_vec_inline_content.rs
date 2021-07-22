use stencila_schema::{
    BlockContent, CodeFragment, InlineContent, MathFragment, Quote, QuoteBlockCite, QuoteCite,
};

pub trait ToVecInlineContent {
    fn to_vec_inline_content(&self) -> Vec<InlineContent>;
}

impl ToVecInlineContent for BlockContent {
    /// Coerce a `BlockContent` node into a vector of `InlineContent` nodes as lossless-ly as possible
    fn to_vec_inline_content(&self) -> Vec<InlineContent> {
        match self {
            // Block content types that have inline content that can be used directly
            BlockContent::Heading(heading) => heading.content.clone(),
            BlockContent::Paragraph(paragraph) => paragraph.content.clone(),

            // Block content types that have inline content analogues
            BlockContent::CodeBlock(code_block) => {
                vec![InlineContent::CodeFragment(CodeFragment {
                    text: code_block.text.clone(),
                    programming_language: code_block.programming_language.clone(),
                    ..Default::default()
                })]
            }
            BlockContent::MathBlock(math_block) => {
                vec![InlineContent::MathFragment(MathFragment {
                    text: math_block.text.clone(),
                    math_language: math_block.math_language.clone(),
                    ..Default::default()
                })]
            }
            BlockContent::QuoteBlock(quote_block) => {
                let content = quote_block.content.to_vec_inline_content();
                let cite = if let Some(cite) = &quote_block.cite {
                    match cite.as_ref() {
                        QuoteBlockCite::Cite(cite) => Some(Box::new(QuoteCite::Cite(cite.clone()))),
                        QuoteBlockCite::String(str) => {
                            Some(Box::new(QuoteCite::String(str.clone())))
                        }
                    }
                } else {
                    None
                };
                vec![InlineContent::Quote(Quote {
                    content,
                    cite,
                    ..Default::default()
                })]
            }

            // Types that have block content to coerce
            BlockContent::Claim(claim) => claim.content.to_vec_inline_content(),

            // Types that have no direct analogue, or are not implemented yet
            // TODO: Implement for other types
            BlockContent::CodeChunk(..)
            | BlockContent::Collection(..)
            | BlockContent::Figure(..)
            | BlockContent::List(..)
            | BlockContent::Table(..)
            | BlockContent::ThematicBreak(..) => vec![],
        }
    }
}

impl ToVecInlineContent for Vec<BlockContent> {
    /// Coerce a vector of `BlockContent` nodes into a vector of `InlineContent` nodes
    fn to_vec_inline_content(&self) -> Vec<InlineContent> {
        self.iter()
            .flat_map(|item| item.to_vec_inline_content())
            .collect()
    }
}
