use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
    },
    format::Format,
    schema::{Node, NodeType},
    status::Status,
    Codec, CodecSupport, EncodeInfo, EncodeOptions,
};
use codec_dom_trait::{
    html_escape::{encode_double_quoted_attribute, encode_safe},
    DomCodec as _, DomEncodeContext,
};
use codec_text_trait::to_text;

/// A codec for DOM HTML
pub struct DomCodec;

#[async_trait]
impl Codec for DomCodec {
    fn name(&self) -> &str {
        "dom"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Dom => CodecSupport::NoLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::NoLoss
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let standalone = options
            .as_ref()
            .and_then(|options| options.standalone)
            .unwrap_or(false);
        let theme = options
            .as_ref()
            .and_then(|options| options.theme.as_deref())
            .unwrap_or("default");
        let compact = options
            .as_ref()
            .and_then(|options| options.compact)
            .unwrap_or(true);
        let source_path = options
            .as_ref()
            .and_then(|options| options.from_path.clone());
        let dest_path = options.as_ref().and_then(|options| options.to_path.clone());

        // Encode to DOM HTML
        let mut context = DomEncodeContext::new(standalone, source_path, dest_path);
        node.to_dom(&mut context);

        // Add the root attribute to the root node (the first opening tag)
        let mut dom = context.content();
        if let Some(pos) = dom.find('>') {
            dom.insert_str(pos, " root");
        }

        // Get any CSS defined in the content (e.g. Tailwind usage, or raw CSS blocks)
        // This needs to be inserted at the top of the root node
        // (for diffing and Morphdom to work it can not go before)
        let css = context.css();
        if !css.is_empty() {
            let css = normalize_css(&css);
            if let Some(pos) = dom.find('>') {
                dom.insert_str(pos + 1, &["<style>", &css, "</style>"].concat());
            }
        }

        let html = if standalone {
            let og_type = format!(
                r#"<meta property="og:type" content="{}" />"#,
                node.node_type()
            );

            let title = match node {
                Node::Article(article) => article.title.as_ref().map(to_text),
                Node::Prompt(prompt) => prompt.id.as_ref().map(to_text),
                _ => None,
            };
            let og_title = title
                .as_ref()
                .map(|title| {
                    format!(
                        r#"<meta property="og:title" content="{}" />"#,
                        encode_double_quoted_attribute(title)
                    )
                })
                .unwrap_or_default();
            let html_title = title.map_or_else(
                || "Untitled".to_string(),
                |title| encode_safe(&title).to_string(),
            );

            let desc = match node {
                Node::Article(article) => article.description.as_ref().map(|cord| cord.to_string()),
                Node::Prompt(prompt) => Some(prompt.description.to_string()),
                _ => None,
            }
            .map(|desc| encode_double_quoted_attribute(&desc).to_string());
            let og_desc = desc
                .as_ref()
                .map(|desc| format!(r#"<meta property="og:description" content="{desc}" />"#,))
                .unwrap_or_default();
            let html_desc = desc
                .map(|desc| format!(r#"<meta property="description" content="{desc}" />"#,))
                .unwrap_or_default();

            let base_url = options
                .as_ref()
                .and_then(|options| options.base_url.as_deref())
                .unwrap_or_default();
            let og_image = context
                .image()
                .as_ref()
                .map(|image| {
                    format!(
                        r#"<meta property="og:image" content="{}" />"#,
                        encode_double_quoted_attribute(&format!("{base_url}/{image}"))
                    )
                })
                .unwrap_or_default();

            let alternates = options
                .as_ref()
                .and_then(|options| options.alternates.clone())
                .iter()
                .flatten()
                .map(|(typ, path)| {
                    format!(r#"<link rel="alternate" type="{typ}" href="{path}" />"#)
                })
                .join("\n    ");

            format!(
                r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8"/>
    <title>{html_title}</title>
    {html_desc}
    {og_type}
    {og_title}
    {og_desc}
    {og_image}
    {alternates}
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="icon" type="image/png" href="/~static/images/favicon.png" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap" rel="stylesheet" />
    <link rel="stylesheet" type="text/css" href="/~static/themes/{theme}.css" />
    <link rel="stylesheet" type="text/css" href="/~static/views/dynamic.css" />
    <script type="module" src="/~static/views/dynamic.js"></script>
  </head>
  <body>
    <stencila-dynamic-view view="dynamic">
      {dom}
    </stencila-dynamic-view>
  </body>
</html>"#
            )
        } else {
            dom
        };

        let html = match compact {
            true => html,
            false => indent_html(&html)?,
        };

        Ok((html, EncodeInfo::none()))
    }
}

/// Indent HTML
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent_html(html: &str) -> Result<String> {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_str(html);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => writer.write_event(event),
            Err(error) => bail!(
                "Error at position {}: {error:?}\n{html}",
                reader.buffer_position()
            ),
        }?;
    }

    Ok(std::str::from_utf8(&writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string())
}

/// Normalize and minify CSS
fn normalize_css(css: &str) -> String {
    StyleSheet::parse(css, ParserOptions::default())
        .map(|stylesheet| {
            stylesheet
                .to_css(PrinterOptions {
                    minify: true,
                    ..Default::default()
                })
                .map(|result| result.code)
                .unwrap_or_else(|_| css.to_string())
        })
        .unwrap_or_else(|_| css.to_string())
}
