use codec::{
    common::eyre::{bail, Result},
    schema::{
        shortcuts::{em, h1, h2, h3, h4, h5, h6, p, sec, stg, stk, sub, sup, t, u},
        Article, Block, ImageObject, Inline, Link, Node, StyledBlock, StyledInline,
    },
    DecodeInfo, DecodeOptions,
};
use tl::{parse, HTMLTag, Parser, ParserOptions, RawChildren};

/// Decode a HTML string to a Stencila Schema [`Node`]
///
/// This is the main entry point for decoding. It parses the HTML, and then traverses the
/// DOM, building an [`Article`] from it (currently HTML is always treated as an article,
/// not any other type of `CreativeWork`).
pub(super) fn decode(html: &str, _options: Option<DecodeOptions>) -> Result<(Node, DecodeInfo)> {
    // Wrap in a <body> if necessary
    let html = if !html.contains("body") {
        ["<body>", html, "</body>"].concat()
    } else {
        html.to_string()
    };

    // Parse the HTML
    let dom = parse(&html, ParserOptions::default())?;
    let parser = dom.parser();

    // Extract the body to decode blocks from
    let Some(body) = dom
        .query_selector("body")
        .and_then(|mut nodes| nodes.next())
        .and_then(|body| body.get(parser))
        .and_then(|body| body.as_tag())
    else {
        bail!("No <body> tag in HTML")
    };

    // Decode blocks and create article
    let content = decode_blocks(parser, body.children().top());

    let node = Node::Article(Article {
        content,
        ..Default::default()
    });

    Ok((node, DecodeInfo::none()))
}

/// Decode block elements
fn decode_blocks(parser: &Parser, nodes: &RawChildren) -> Vec<Block> {
    let mut blocks = Vec::new();
    for child in nodes.iter().flat_map(|handle| handle.get(parser)) {
        let block = if let Some(tag) = child.as_tag() {
            let name = tag.name().as_utf8_str();
            match name.as_ref() {
                "div" => decode_div(parser, tag),
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => decode_h(parser, tag, &name),
                "p" => decode_p(parser, tag),
                // Inlines where block is expected
                "a" => p([decode_a(parser, tag)]),
                "img" => p([decode_img(parser, tag)]),
                "span" => p(decode_span(parser, tag)),
                // Unhandled tag: just decode children into blocks
                _ => {
                    blocks.append(&mut decode_blocks(parser, tag.children().top()));
                    continue;
                }
            }
        } else if let Some(text) = child.as_raw() {
            // At block level, ignore whitespace
            let text = text.try_as_utf8_str().unwrap_or_default().trim();
            if !text.is_empty() {
                p([t(text)])
            } else {
                continue;
            }
        } else {
            continue;
        };
        blocks.push(block);
    }
    blocks
}

/// Decode a <div> element into either a [`StyledBlock`] or [`Section`] node
fn decode_div(parser: &Parser, tag: &HTMLTag) -> Block {
    let attrs = tag.attributes();
    let content = decode_blocks(parser, tag.children().top());
    if let Some(classes) = attrs.class() {
        Block::StyledBlock(StyledBlock {
            code: classes.as_utf8_str().into(),
            style_language: Some("tailwind".to_string()),
            content,
            ..Default::default()
        })
    } else if let Some(style) = attrs.get("style").flatten() {
        Block::StyledBlock(StyledBlock {
            code: style.as_utf8_str().into(),
            style_language: Some("css".to_string()),
            content,
            ..Default::default()
        })
    } else {
        sec(content)
    }
}

/// Decode a <h1>, <h2>,... element into a [`Heading`]
fn decode_h(parser: &Parser, tag: &HTMLTag, name: &str) -> Block {
    let content = decode_inlines(parser, tag.children().top());
    match name {
        "h1" => h1(content),
        "h2" => h2(content),
        "h3" => h3(content),
        "h4" => h4(content),
        "h5" => h5(content),
        _ => h6(content),
    }
}

/// Decode a <p> element into a [`Paragraph`]
fn decode_p(parser: &Parser, tag: &HTMLTag) -> Block {
    p(decode_inlines(parser, tag.children().top()))
}

/// Decode inline elements
fn decode_inlines(parser: &Parser, nodes: &RawChildren) -> Vec<Inline> {
    let mut inlines = Vec::new();
    for child in nodes.iter().flat_map(|handle| handle.get(parser)) {
        let inline = if let Some(tag) = child.as_tag() {
            let name = tag.name().as_utf8_str();
            match name.as_ref() {
                "em" | "i" | "strong" | "bold" | "u" | "sub" | "sup" | "s" => {
                    decode_mark(parser, tag, &name)
                }
                "a" => decode_a(parser, tag),
                "img" => decode_img(parser, tag),
                "span" => {
                    inlines.append(&mut decode_span(parser, tag));
                    continue;
                }
                // Unhandled tag: just decode children into inlines
                _ => {
                    inlines.append(&mut decode_inlines(parser, tag.children().top()));
                    continue;
                }
            }
        } else if let Some(text) = child.as_raw() {
            // Trim inline text, but no smaller than a space
            let mut text = text.try_as_utf8_str().unwrap_or_default().trim();
            if text.is_empty() {
                text = " ";
            }
            t(text)
        } else {
            continue;
        };
        inlines.push(inline);
    }
    inlines
}

/// Decode a simple inline "mark" element
fn decode_mark(parser: &Parser, tag: &HTMLTag, name: &str) -> Inline {
    let content = decode_inlines(parser, tag.children().top());
    match name {
        "em" | "i" => em(content),
        "strong" | "bold" => stg(content),
        "u" => u(content),
        "sup" => sup(content),
        "sub" => sub(content),
        "s" => stk(content),
        _ => em(content),
    }
}

/// Decode an <a> element into a [`Link`] node
fn decode_a(parser: &Parser, tag: &HTMLTag) -> Inline {
    let attrs = tag.attributes();
    let target = attrs
        .get("href")
        .flatten()
        .map(|bytes| bytes.as_utf8_str())
        .unwrap_or_default()
        .to_string();
    let title = attrs
        .get("title")
        .flatten()
        .map(|bytes| bytes.as_utf8_str().to_string());

    let content = decode_inlines(parser, tag.children().top());

    Inline::Link(Link {
        target,
        content,
        title,
        ..Default::default()
    })
}

/// Decode a <img> element into a [`ImageObject`] node
fn decode_img(_parser: &Parser, tag: &HTMLTag) -> Inline {
    let attrs = tag.attributes();
    let content_url = attrs
        .get("src")
        .flatten()
        .map(|bytes| bytes.as_utf8_str())
        .unwrap_or_default()
        .to_string();
    let caption = attrs
        .get("alt")
        .flatten()
        .map(|bytes| bytes.as_utf8_str().to_string())
        .map(|alt| vec![t(alt)]);
    let title = attrs
        .get("title")
        .flatten()
        .map(|bytes| bytes.as_utf8_str().to_string())
        .map(|alt| vec![t(alt)]);

    Inline::ImageObject(ImageObject {
        content_url,
        caption,
        title,
        ..Default::default()
    })
}

/// Decode a <span> element into either a [`StyledInline`] or [`Text`] node
fn decode_span(parser: &Parser, tag: &HTMLTag) -> Vec<Inline> {
    let attrs = tag.attributes();
    let content = decode_inlines(parser, tag.children().top());
    if let Some(classes) = attrs.class() {
        vec![Inline::StyledInline(StyledInline {
            code: classes.as_utf8_str().into(),
            style_language: Some("tailwind".to_string()),
            content,
            ..Default::default()
        })]
    } else if let Some(style) = attrs.get("style").flatten() {
        vec![Inline::StyledInline(StyledInline {
            code: style.as_utf8_str().into(),
            style_language: Some("css".to_string()),
            content,
            ..Default::default()
        })]
    } else {
        content
    }
}
