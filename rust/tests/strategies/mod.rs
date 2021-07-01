///! Property-based testing strategies
use itertools::interleave;
use proptest::collection::{size_range, vec};
use proptest::prelude::*;
use proptest::strategy::Union;
use stencila_schema::{
    Article, AudioObjectSimple, BlockContent, CodeBlock, CodeFragment, Emphasis, Heading,
    ImageObjectSimple, InlineContent, Link, List, ListItem, ListItemContent, ListOrder, Node,
    Paragraph, Strong, ThematicBreak, VideoObjectSimple,
};

/// The degree of freedom when generating arbitrary nodes.
///
/// Generally, when adding a `proptest` it is wise to start with `Nil`
/// freedom and gradually increase it while fixing issues along the way.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Freedom {
    Nil,
    Low,
    High,
}

prop_compose! {
    /// Generate an arbitrary inline string
    pub fn string(freedom: Freedom)(
        string in (match freedom {
            Freedom::Nil => r"string",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        }).prop_filter(
            "Inline strings should not be empty",
            |string| !string.is_empty()
        )
    ) -> InlineContent {
        InlineContent::String(string)
    }
}

prop_compose! {
    /// Generate an arbitrary audio object
    pub fn audio_object_simple(freedom: Freedom)(
        content_url in match freedom {
            Freedom::Nil => r"url",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        }
    ) -> InlineContent {
        InlineContent::AudioObject(AudioObjectSimple{
            content_url,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate an arbitrary image object
    pub fn image_object_simple(freedom: Freedom)(
        content_url in match freedom {
            Freedom::Nil => r"url",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        }
    ) -> InlineContent {
        InlineContent::ImageObject(ImageObjectSimple{
            content_url,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate an arbitrary video object
    pub fn video_object_simple(freedom: Freedom)(
        content_url in match freedom {
            Freedom::Nil => r"url",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        }
    ) -> InlineContent {
        InlineContent::VideoObject(VideoObjectSimple{
            content_url,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a code fragment node with arbitrary text and programming language
    pub fn code_fragment(freedom: Freedom)(
        text in match freedom {
            Freedom::Nil => r"text",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        },
        programming_language in match freedom {
            Freedom::Nil => r"lang",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        }
    ) -> InlineContent {
        let programming_language = if programming_language.is_empty() {
            None
        } else {
            Some(Box::new(programming_language))
        };
        InlineContent::CodeFragment(CodeFragment{
            text,
            programming_language,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a emphasis node with arbitrary content
    pub fn emphasis(freedom: Freedom)(
        content in string(freedom)
    ) -> InlineContent {
        InlineContent::Emphasis(Emphasis{
            content:vec![content],
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a strong node with arbitrary content
    pub fn strong(freedom: Freedom)(
        content in string(freedom)
    ) -> InlineContent {
        InlineContent::Strong(Strong{
            content:vec![content],
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a link with arbitrary target and content
    pub fn link(freedom: Freedom)(
        target in match freedom {
            Freedom::Nil => r"target",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        },
        content in string(freedom)
    ) -> InlineContent {
        InlineContent::Link(Link{
            target,
            content:vec![content],
            ..Default::default()
        })
    }
}

/// Generate one of the inline content node types excluding strings (which
/// we usually want to be interleaved between them).
pub fn inline_content(freedom: Freedom) -> impl Strategy<Value = InlineContent> {
    Union::new(vec![
        audio_object_simple(freedom).boxed(),
        image_object_simple(freedom).boxed(),
        video_object_simple(freedom).boxed(),
        code_fragment(freedom).boxed(),
        emphasis(freedom).boxed(),
        strong(freedom).boxed(),
        link(freedom).boxed(),
    ])
}

prop_compose! {
    /// Generate a vector of inline content of arbitrary length and content
    /// but always having strings interspersed by other inline content (to separate them).
    pub fn vec_inline_content(freedom: Freedom)(length in 1usize..10)(
        strings in vec(string(freedom), size_range(length)),
        others in vec(inline_content(freedom), size_range(length))
    ) -> Vec<InlineContent> {
        interleave(strings, others).collect()
    }
}

prop_compose! {
    /// Generate a code block node with arbitrary text and programming language
    pub fn code_block(freedom: Freedom)(
        text in match freedom {
            Freedom::Nil => r"text",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        },
        programming_language in match freedom {
            Freedom::Nil => r"lang",
            Freedom::Low => r"[a-zA-Z0-9 \t\n]+",
            Freedom::High => any::<String>()
        }
    ) -> BlockContent {
        let programming_language = if programming_language.is_empty() {
            None
        } else {
            Some(Box::new(programming_language))
        };
        BlockContent::CodeBlock(CodeBlock{
            text,
            programming_language,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a heading with arbitrary content and depth
    pub fn heading(freedom: Freedom)(
        depth in 1..6,
        content in vec_inline_content(freedom)
    ) -> BlockContent {
        BlockContent::Heading(Heading{
            depth: Some(Box::new(depth as i64)),
            content,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a paragraph with arbitrary content
    pub fn paragraph(freedom: Freedom)(
        content in vec_inline_content(freedom)
    ) -> BlockContent {
        BlockContent::Paragraph(Paragraph{
            content,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a list with arbitrary items and order
    pub fn list(freedom: Freedom)(
        order in prop_oneof![Just(ListOrder::Ascending), Just(ListOrder::Unordered)],
        items in vec(list_item(freedom), 1..5)
    ) -> BlockContent {
        BlockContent::List(List{
            order: Some(Box::new(order)),
            items,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a list item with arbitrary inline content.
    /// Unable to use block_content here because that causes infinite recursion
    pub fn list_item(freedom: Freedom)(
        content in vec(inline_content(freedom), 1..3)
    ) -> ListItem {
        ListItem{
            content: Some(Box::new(ListItemContent::VecInlineContent(content))),
            ..Default::default()
        }
    }
}

/// Generate a thematic break
pub fn thematic_break() -> impl Strategy<Value = BlockContent> {
    Just(BlockContent::ThematicBreak(ThematicBreak::default()))
}

/// Generate one of the block content node types
pub fn block_content(freedom: Freedom) -> impl Strategy<Value = BlockContent> {
    Union::new(vec![
        code_block(freedom).boxed(),
        heading(freedom).boxed(),
        list(freedom).boxed(),
        paragraph(freedom).boxed(),
        thematic_break().boxed(),
    ])
}

prop_compose! {
    /// Generate an article with arbitrary content (and in the future, other properties)
    pub fn article(freedom: Freedom)(
        content in vec(block_content(freedom), 1..10)
    ) -> Node {
        Node::Article(Article{content: Some(content), ..Default::default()})
    }
}

/// Generate an arbitrary node
pub fn node(freedom: Freedom) -> impl Strategy<Value = Node> {
    Union::new(vec![article(freedom).boxed()])
}
