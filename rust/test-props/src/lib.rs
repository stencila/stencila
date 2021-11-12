///! Utility functions and strategies for property testing
use itertools::interleave;
use proptest::collection::{size_range, vec};
use proptest::prelude::*;
use proptest::strategy::Union;
use stencila_schema::*;

// Export proptest for use in other internal crates
pub use proptest;

/// The degree of freedom when generating arbitrary nodes.
///
/// Generally, when adding a `proptest` it is wise to start with `Nil`
/// freedom and gradually increase it while fixing issues along the way.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Freedom {
    Min,
    Low,
    High,
    Max,
}

prop_compose! {
    /// Generate an arbitrary inline string
    pub fn string(freedom: Freedom)(
        string in (match freedom {
            Freedom::Min => r"string",
            Freedom::Low => r"[A-Za-z0-9 ]+",
            _ => any::<String>(),
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
            Freedom::Min => r"string",
            _ => r"[A-Za-z0-9]+",
        }
    ) -> InlineContent {
        InlineContent::String(string)
    }
}

prop_compose! {
    /// Generate inline content for inside other inline content
    pub fn inline_inner_content(freedom: Freedom)(
        string in (match freedom {
            Freedom::Min => r"string",
            Freedom::Low => r"[A-Za-z0-9]+", // Note: no whitespace or "special" characters
            _ => any::<String>(),
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
    /// Use audio file extensions because Markdown decoding uses that to determine
    /// to decode to a `AudioObject`.
    pub fn audio_object_simple(freedom: Freedom)(
        content_url in match freedom {
            Freedom::Min => r"url\.mp3",
            Freedom::Low => r"[A-Za-z0-9-_]+\.(flac|mp3|ogg)",
            _ => r"\PC*\.(flac|mp3|ogg)",
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
            Freedom::Min => r"url\.png",
            Freedom::Low => r"[A-Za-z0-9-_]\.(gif|jpg|jpeg|png)",
            _ => r"\PC*\.(gif|jpg|jpeg|png)",
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
            Freedom::Min => r"url\.mp4",
            Freedom::Low => r"[A-Za-z0-9-_]\.(3gp|mp4|ogv|webm)",
            _ => r"\PC*\.(3gp|mp4|ogv|webm)",
        }
    ) -> InlineContent {
        InlineContent::VideoObject(VideoObjectSimple{
            content_url,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a code expression node with arbitrary text and programming language
    ///
    /// With `Freedom::Low` only allow language codes that are recognized when decoding
    /// formats such as R Markdown.
    pub fn code_expression(freedom: Freedom)(
        programming_language in match freedom {
            Freedom::Min => "py",
            Freedom::Low => r"js|py|r",
            Freedom::High => r"[A-Za-z0-9-]+",
            _ => any::<String>()
        },
        text in match freedom {
            Freedom::Min => "text",
            Freedom::Low => r"[A-Za-z0-9-_ ]+",
            _ => any::<String>()
        },
    ) -> InlineContent {
        InlineContent::CodeExpression(CodeExpression{
            text,
            programming_language,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a code fragment node with arbitrary text and programming language
    pub fn code_fragment(freedom: Freedom)(
        text in match freedom {
            Freedom::Min => r"text",
            Freedom::Low => r"[A-Za-z0-9-_ ]+",
            _ => any::<String>()
        },
        programming_language in match freedom {
            Freedom::Min => "",
            Freedom::Low => r"[A-Za-z0-9-]+",
            _ => any::<String>()
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
        content in inline_inner_content(freedom)
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
        content in inline_inner_content(freedom)
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
            Freedom::Min => r"target",
            Freedom::Low => r"[A-Za-z0-9-]*",
            _ => any::<String>()
        },
        content in inline_inner_content(freedom)
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
        content in inline_inner_content(freedom)
    ) -> InlineContent {
        InlineContent::NontextualAnnotation(NontextualAnnotation{
            content:vec![content],
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a quote node with arbitrary content
    pub fn quote(freedom: Freedom)(
        content in inline_inner_content(freedom)
    ) -> InlineContent {
        InlineContent::Quote(Quote{
            content:vec![content],
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a strong node with arbitrary content
    pub fn strong(freedom: Freedom)(
        content in inline_inner_content(freedom)
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
        content in inline_inner_content(freedom)
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
        content in inline_inner_content(freedom)
    ) -> InlineContent {
        InlineContent::Superscript(Superscript{
            content:vec![content],
            ..Default::default()
        })
    }
}

/// Generate one of the inline content node types excluding strings (which
/// we usually want to be interleaved between them).
pub fn inline_content(
    freedom: Freedom,
    exclude_types: Vec<String>,
) -> impl Strategy<Value = InlineContent> {
    let mut types = Vec::new();
    for (name, strategy) in [
        ("AudioObject", audio_object_simple(freedom).boxed()),
        ("ImageObject", image_object_simple(freedom).boxed()),
        ("VideoObject", video_object_simple(freedom).boxed()),
        ("CodeExpression", code_expression(freedom).boxed()),
        ("CodeFragment", code_fragment(freedom).boxed()),
        ("Delete", delete(freedom).boxed()),
        ("Emphasis", emphasis(freedom).boxed()),
        ("Link", link(freedom).boxed()),
        (
            "NontextualAnnotation",
            nontextual_annotation(freedom).boxed(),
        ),
        ("Quote", quote(freedom).boxed()),
        ("Strong", strong(freedom).boxed()),
        ("Subscript", subscript(freedom).boxed()),
        ("Superscript", superscript(freedom).boxed()),
    ] {
        if !exclude_types.contains(&name.to_string()) {
            types.push(strategy)
        }
    }
    Union::new(types)
}

prop_compose! {
    /// Generate a vector of inline content of arbitrary length and content
    /// but always having strings interspersed by other inline content (to separate them
    /// so that they do not get decoded as a single string).
    ///
    /// Restrictions:
    ///   - Always starts and ends with a string.
    ///   - Ensures that nodes such as `Strong` and `Emphasis` are surrounded by spaces (for Markdown).
    ///   - No leading or trailing whitespace (for Markdown).
    pub fn vec_inline_content(freedom: Freedom, exclude_properties: Vec<String>)(
        length in 1usize..(match freedom {
            Freedom::Min => 1,
            Freedom::Low => 5,
            _ => 10,
        } + 1)
    )(
        strings in vec(string(freedom), size_range(length + 1)),
        others in vec(inline_content(freedom, exclude_properties.clone()), size_range(length))
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
                    if string.starts_with(char::is_whitespace) {
                        string.insert(0, 'A')
                    }
                }
            }
            if index == content.len() - 1 {
                if let InlineContent::String(string) = &mut content[index] {
                    if string.ends_with(char::is_whitespace) {
                        string.push('.')
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
            Freedom::Min => r"text",
            Freedom::Low => r"[A-Za-z0-9-_ \t\n]*",
            _ => any::<String>()
        },
        programming_language in match freedom {
            Freedom::Min => "",
            Freedom::Low => r"[A-Za-z0-9-]*",
            _ => any::<String>()
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
    pub fn heading(freedom: Freedom, exclude_types: Vec<String>)(
        depth in 1..6,
        content in match freedom {
            Freedom::Min => vec(string(freedom), 1..2).boxed(),
            _ => vec_inline_content(freedom, exclude_types).boxed()
        }
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
    pub fn paragraph(freedom: Freedom, exclude_types: Vec<String>)(
        content in vec_inline_content(freedom, exclude_types)
    ) -> BlockContent {
        BlockContent::Paragraph(Paragraph{
            content,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a list with arbitrary items and order
    pub fn list(freedom: Freedom, exclude_types: Vec<String>)(
        order in prop_oneof![Just(ListOrder::Ascending), Just(ListOrder::Unordered)],
        items in vec(list_item(freedom, exclude_types), 1..(match freedom {
            Freedom::Min => 1,
            Freedom::Low => 3,
            _ => 5,
        } + 1))
    ) -> BlockContent {
        BlockContent::List(List{
            order: Some(order),
            items,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a list item with arbitrary block content.
    /// Unable to use block_content strategy here because that causes infinite recursion.
    /// Be careful increasing the length of content as that can slow down test
    /// significantly (given that this is an inner "loop").
    pub fn list_item(freedom: Freedom, exclude_types: Vec<String>)(
        content in vec(Union::new(vec![
            paragraph(freedom, exclude_types).boxed(),
        ]), 1..(match freedom {
            Freedom::Min => 1,
            Freedom::Low => 2,
            _ => 3,
        } + 1))
    ) -> ListItem {
        ListItem{
            content: Some(ListItemContent::VecBlockContent(content)),
            ..Default::default()
        }
    }
}

prop_compose! {
    /// Generate a quote block with arbitrary block content.
    /// Does no allow for quote blocks (because that would be a recursive
    /// strategy), or lists or thematic breaks (because they need filtering, see below)
    pub fn quote_block(freedom: Freedom, exclude_types: Vec<String>)(
        content in vec(Union::new(
            match freedom {
                Freedom::Min => vec![
                    paragraph(freedom, exclude_types).boxed(),
                ],
                _ => vec![
                    code_block(freedom).boxed(),
                    heading(freedom, exclude_types.clone()).boxed(),
                    paragraph(freedom, exclude_types).boxed(),
                ]
            }),
            1..(match freedom {
                Freedom::Min => 1,
                Freedom::Low => 3,
                _ => 5,
            } + 1)
        )
    ) -> BlockContent {
        BlockContent::QuoteBlock(QuoteBlock{
            content,
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a table.
    pub fn table(freedom: Freedom)(
        head in vec(table_row(freedom, Some(TableRowRowType::Header), Some(TableCellCellType::Header)), 1..(match freedom {
            Freedom::Max => 3,
            // Markdown only supports a single header row
            _ => 1,
        } + 1)),
        body in vec(table_row(freedom, None, None), 1..(match freedom {
            Freedom::Min => 1,
            Freedom::Low => 5,
            _ => 10,
        } + 1))
    ) -> BlockContent {
        BlockContent::Table(TableSimple{
            rows: [head, body].concat(),
            ..Default::default()
        })
    }
}

prop_compose! {
    /// Generate a table row.
    pub fn table_row(freedom: Freedom, row_type: Option<TableRowRowType>, cell_type: Option<TableCellCellType>)(
        cells in vec(table_cell(freedom, cell_type), 1..(match freedom {
            Freedom::Min => 1,
            Freedom::Low => 5,
            _ => 10,
        } + 1))
    ) -> TableRow {
        TableRow{
            cells,
            row_type: row_type.clone(),
            ..Default::default()
        }
    }
}

prop_compose! {
    /// Generate a table cell.
    pub fn table_cell(freedom: Freedom, cell_type: Option<TableCellCellType>)(
        content in string(freedom)
    ) -> TableCell {
        TableCell{
            content: Some(TableCellContent::VecInlineContent(vec![content])),
            cell_type: cell_type.clone(),
            ..Default::default()
        }
    }
}

prop_compose! {
    /// Generate a code chunk
    ///
    /// With `Freedom::Low` only allow language codes that are recognized when decoding
    /// formats such as R Markdown.
    pub fn code_chunk(freedom: Freedom)(
        programming_language in match freedom {
            Freedom::Min => "py",
            Freedom::Low => r"js|py|r",
            Freedom::High => r"[A-Za-z0-9-]+",
            _ => any::<String>()
        },
        text in match freedom {
            Freedom::Min => "text",
            Freedom::Low => r"[A-Za-z0-9-_ ]+",
            _ => any::<String>()
        }
    ) -> BlockContent {
        BlockContent::CodeChunk(CodeChunk{
            programming_language,
            text,
            ..Default::default()
        })
    }
}

/// Generate a thematic break
pub fn thematic_break() -> impl Strategy<Value = BlockContent> {
    Just(BlockContent::ThematicBreak(ThematicBreak::default()))
}

/// Generate one of the block content node types
pub fn block_content(
    freedom: Freedom,
    exclude_types: Vec<String>,
) -> impl Strategy<Value = BlockContent> {
    let mut strategies = Vec::new();
    if !exclude_types.contains(&"CodeBlock".to_string()) {
        strategies.push(code_block(freedom).boxed())
    }
    if !exclude_types.contains(&"CodeChunk".to_string()) {
        strategies.push(code_chunk(freedom).boxed())
    }
    if !exclude_types.contains(&"Heading".to_string()) {
        strategies.push(heading(freedom, exclude_types.clone()).boxed())
    }
    if !exclude_types.contains(&"List".to_string()) {
        strategies.push(list(freedom, exclude_types.clone()).boxed())
    }
    if !exclude_types.contains(&"Paragraph".to_string()) {
        strategies.push(paragraph(freedom, exclude_types.clone()).boxed())
    }
    if !exclude_types.contains(&"QuoteBlock".to_string()) {
        strategies.push(quote_block(freedom, exclude_types.clone()).boxed())
    }
    if !exclude_types.contains(&"Table".to_string()) {
        strategies.push(table(freedom).boxed())
    }
    if !exclude_types.contains(&"ThematicBreak".to_string()) {
        strategies.push(thematic_break().boxed())
    }
    Union::new(strategies)
}

prop_compose! {
    /// Generate a vector of block content of arbitrary length and content
    ///
    /// Restrictions:
    ///  - Does not start with a thematic break (unrealistic, and in Markdown can
    ///    be confused with YAML frontmatter)
    ///  - List of same ordering can not be adjacent to each other (in Markdown they
    ///    get decoded as the same list)
    pub fn vec_block_content(freedom: Freedom, exclude_types: Vec<String>)(
        length in 1usize..(match freedom {
            Freedom::Min => 1,
            Freedom::Low => 5,
            _ => 10,
        } + 1)
    )(
        blocks in vec(block_content(freedom, exclude_types.clone()), size_range(length))
            .prop_filter(
                "Should not start with thematic break",
                |blocks| {
                    !(matches!(blocks[0], BlockContent::ThematicBreak(..)))
            })
            .prop_filter(
                "Lists with same ordering should not be adjacent",
                |blocks| {
                    for index in 1..blocks.len() {
                        if let (BlockContent::List(prev), BlockContent::List(curr)) = (&blocks[index-1], &blocks[index]) {
                            match (&prev.order, &curr.order) {
                                (None, None) |
                                (Some(ListOrder::Ascending), Some(ListOrder::Ascending)) |
                                (Some(ListOrder::Unordered), Some(ListOrder::Unordered)) => {
                                    return false
                                },
                                _ => ()
                            }
                        }
                    }
                    true
                }
            )
    ) -> Vec<BlockContent> {
        blocks
    }
}

prop_compose! {
    /// Generate an article with arbitrary content (and in the future, other properties)
    pub fn article(freedom: Freedom, exclude_types: Vec<String>, _exclude_properties: Vec<String>)(
        content in vec_block_content(freedom, exclude_types)
    ) -> Node {
        Node::Article(Article{content: Some(content), ..Default::default()})
    }
}

/// Generate an arbitrary node
pub fn node(freedom: Freedom) -> impl Strategy<Value = Node> {
    Union::new(vec![article(freedom, vec![], vec![]).boxed()])
}
