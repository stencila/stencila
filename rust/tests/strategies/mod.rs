///! Property testing strategies
use itertools::interleave;
use proptest::collection::{size_range, vec};
use proptest::prelude::*;
use proptest::strategy::Union;
use stencila_schema::{
    Article, BlockContent, CodeFragment, Emphasis, InlineContent, Link, Node, Paragraph,
};

prop_compose! {
    /// Generate a random string
    pub fn string()(
        string in r"[a-z]+" // TODO: loosen
    ) -> InlineContent {
        InlineContent::String(string)
    }
}

prop_compose! {
    /// Generate a code fragment node with random text and programming language
    pub fn code_fragment()(
        text: String,
        programming_language: String
    ) -> InlineContent {
        let programming_language = if programming_language.is_empty() {
            None
        } else {
            Some(Box::new(programming_language))
        };
        InlineContent::CodeFragment(CodeFragment{text, programming_language, ..Default::default()})
    }
}

prop_compose! {
    /// Generate a emphasis node with random content
    pub fn emphasis()(
        content in string()
    ) -> InlineContent {
        InlineContent::Emphasis(Emphasis{content:vec![content], ..Default::default()})
    }
}

prop_compose! {
    /// Generate a link with random target and content
    pub fn link()(
        target in r"[\w]*", // TODO: loosen
        content in string()
    ) -> InlineContent {
        InlineContent::Link(Link{target, content:vec![content], ..Default::default()})
    }
}

/// Generate one of the inline content node types excluding strings (which
/// we usually want to be interleaved between them).
fn inline_content() -> impl Strategy<Value = InlineContent> {
    Union::new(vec![
        // TODO reinstate code_fragment().boxed(),
        emphasis().boxed(),
        link().boxed(),
    ])
}

prop_compose! {
    /// Generate a vector of inline content of random length and content
    /// but always having strings interspersed by other inline content (to separate them).
    pub fn vec_inline_content()(length in 1usize..10)(
        strings in vec(string(), size_range(length)),
        others in vec(inline_content(), size_range(length))
    ) -> Vec<InlineContent> {
        interleave(strings, others).collect()
    }
}

prop_compose! {
    /// Generate a paragraph with random content
    pub fn paragraph()(
        content in vec_inline_content()
    ) -> BlockContent {
        BlockContent::Paragraph(Paragraph{content, ..Default::default()})
    }
}

prop_compose! {
    /// Generate one of the block content node types.
    pub fn block_content()(
        paragraph in paragraph()
    ) -> BlockContent {
        prop_oneof![
            paragraph
        ]
    }
}

prop_compose! {
    /// Generate an article with random content (and in the future, other properties)
    pub fn article()(
        content in vec(block_content(), 0..10)
    ) -> Node {
        Node::Article(Article{content: Some(content), ..Default::default()})
    }
}
