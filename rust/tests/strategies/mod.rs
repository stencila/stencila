///! Property-based testing strategies
use itertools::interleave;
use proptest::collection::{size_range, vec};
use proptest::prelude::*;
use proptest::strategy::Union;
use stencila_schema::*;

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
            Freedom::Low => r"[A-Za-z0-9 ]+",
            Freedom::High => any::<String>(),
        }).prop_filter(
            "Inline strings should not be empty",
            |string| !string.is_empty()
        )
    ) -> InlineContent {
        InlineContent::String(string)
    }
}

prop_compose! {
    /// Generate an arbitrary inline string with no spaces
    pub fn string_no_whitespace(freedom: Freedom)(
        string in match freedom {
            Freedom::Nil => r"string",
            _ => r"[A-Za-z0-9]+",
        }
    ) -> InlineContent {
        InlineContent::String(string)
    }
}

prop_compose! {
    /// Generate an arbitrary audio object
    /// Use audio file extensions because Markdown decoding uses that to determine
    /// to decode to a `AudioObject`.
    pub fn audio_object_simple(freedom: Freedom)(
        content_url in match freedom {
            Freedom::Nil => r"url\.mp3",
            Freedom::Low => r"[A-Za-z0-9-_]+\.(flac|mp3|ogg)",
            Freedom::High => r"\PC*\.(flac|mp3|ogg)",
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
    /// Use image file extensions because Markdown decoding uses that to determine
    /// to decode to a `ImageObject`.
    pub fn image_object_simple(freedom: Freedom)(
        content_url in match freedom {
            Freedom::Nil => r"url\.png",
            Freedom::Low => r"[A-Za-z0-9-_]\.(gif|jpg|jpeg|png)",
            Freedom::High => r"\PC*\.(gif|jpg|jpeg|png)",
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
    /// Use video file extensions because Markdown decoding uses that to determine
    /// to decode to a `VideoObject`.
    pub fn video_object_simple(freedom: Freedom)(
        content_url in match freedom {
            Freedom::Nil => r"url\.mp4",
            Freedom::Low => r"[A-Za-z0-9-_]\.(3gp|mp4|ogv|webm)",
            Freedom::High => r"\PC*\.(3gp|mp4|ogv|webm)",
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
            Freedom::Low => r"[A-Za-z0-9-_ ]+",
            Freedom::High => any::<String>()
        },
        programming_language in match freedom {
            Freedom::Nil => "",
            Freedom::Low => r"[A-Za-z0-9-]+",
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
    /// Generate a delete node with arbitrary content
    pub fn delete(freedom: Freedom)(
        content in string_no_whitespace(freedom)
    ) -> InlineContent {
        InlineContent::Delete(Delete{
            content:vec![content],
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a emphasis node with arbitrary content
    pub fn emphasis(freedom: Freedom)(
        content in string_no_whitespace(freedom)
    ) -> InlineContent {
        InlineContent::Emphasis(Emphasis{
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
            Freedom::Low => r"[A-Za-z0-9-]*",
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

prop_compose! {
    /// Generate a nontextual annotation node with arbitrary content
    pub fn nontextual_annotation(freedom: Freedom)(
        content in string(freedom)
    ) -> InlineContent {
        InlineContent::NontextualAnnotation(NontextualAnnotation{
            content:vec![content],
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a strong node with arbitrary content
    pub fn strong(freedom: Freedom)(
        content in string_no_whitespace(freedom)
    ) -> InlineContent {
        InlineContent::Strong(Strong{
            content:vec![content],
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a subscript node with arbitrary content
    pub fn subscript(freedom: Freedom)(
        content in string_no_whitespace(freedom)
    ) -> InlineContent {
        InlineContent::Subscript(Subscript{
            content:vec![content],
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a superscript node with arbitrary content
    pub fn superscript(freedom: Freedom)(
        content in string_no_whitespace(freedom)
    ) -> InlineContent {
        InlineContent::Superscript(Superscript{
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
        delete(freedom).boxed(),
        emphasis(freedom).boxed(),
        link(freedom).boxed(),
        nontextual_annotation(freedom).boxed(),
        strong(freedom).boxed(),
        subscript(freedom).boxed(),
        superscript(freedom).boxed(),
    ])
}

prop_compose! {
    /// Generate a vector of inline content of arbitrary length and content
    /// but always having strings interspersed by other inline content (to separate them
    /// so that they do not get decoded as a single string).
    ///
    /// Always starts and ends with a string.  For Markdown compatibility, ensures that nodes
    /// such as `Strong` and `Emphasis` are surrounded by spaces and that there is no
    /// leading or trailing blank strings.
    pub fn vec_inline_content(freedom: Freedom)(length in 1usize..10)(
        strings in vec(string(freedom), size_range(length + 1)),
        others in vec(inline_content(freedom), size_range(length))
    ) -> Vec<InlineContent> {
        let mut content: Vec<InlineContent> = interleave(strings, others).collect();
        for index in 0..content.len() {
            let spaces = match content[index] {
                InlineContent::Emphasis(..) | InlineContent::Strong(..) | InlineContent::Delete(..) => {
                   true
                },
                _ => false
            };

            if spaces {
                if let InlineContent::String(string) = &mut content[index - 1] {
                    *string = [string.as_str(), " "].concat();
                }
                if let InlineContent::String(string) = &mut content[index + 1] {
                    *string = [" ", string.as_str()].concat();
                }
            }

            if index == 0 {
                if let InlineContent::String(string) = &mut content[index] {
                    if string.trim().is_empty() {
                        *string = "Unblanked".to_string();
                    }
                }
            }
            if index == content.len() - 1 {
                if let InlineContent::String(string) = &mut content[index] {
                    if string.trim().is_empty() {
                        *string = ".".to_string();
                    }
                }
            }
        }
        content
    }
}

prop_compose! {
    /// Generate a code block node with arbitrary text and programming language
    pub fn code_block(freedom: Freedom)(
        text in match freedom {
            Freedom::Nil => r"text",
            Freedom::Low => r"[A-Za-z0-9-_ \t\n]*",
            Freedom::High => any::<String>()
        },
        programming_language in match freedom {
            Freedom::Nil => "",
            Freedom::Low => r"[A-Za-z0-9-]*",
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
            depth: Some(depth as u8),
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
            order: Some(order),
            items,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a list item with arbitrary inline content.
    /// Unable to use block_content strategy here because that causes infinite recursion.
    /// Instead, currently, allow for an single paragraph
    // TODO: allow for alternative numbers of paragraphs
    pub fn list_item(freedom: Freedom)(
        content in vec_inline_content(freedom)
    ) -> ListItem {
        let content = Some(
            ListItemContent::VecBlockContent(vec![
                BlockContent::Paragraph(
                    Paragraph{
                        content,
                        ..Default::default()
                    }
                )
            ])
        );

        ListItem{
            content,
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
