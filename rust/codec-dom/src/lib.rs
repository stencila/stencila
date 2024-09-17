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
        Status::UnderDevelopment
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
        let compact = options
            .as_ref()
            .and_then(|options| options.compact)
            .unwrap_or(true);

        // Encode to DOM HTML
        let mut context = DomEncodeContext::new(standalone);
        node.to_dom(&mut context);

        // Add the root attribute to the root node (the first opening tag)
        let mut dom = context.content();
        if let Some(pos) = dom.find('>') {
            dom.insert_str(pos, " root");
        }

        // Get any styles defined in the content (e.g. Tailwind usage, or raw CSS blocks)
        // If not standalone then this needs to be inserted at the top of the root node
        // (for diffing and Morphdom to work it can not go before)
        let style = context.style();
        if !standalone && !style.is_empty() {
            if let Some(pos) = dom.find('>') {
                dom.insert_str(pos + 1, &style);
            }
        }

        let html = if standalone {
            let og_type = format!(
                r#"<meta property="og:type" content="{}" />"#,
                node.node_type().to_string()
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
                Node::Article(article) => article
                    .options
                    .description
                    .as_ref()
                    .map(|cord| cord.to_string()),
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

            let og_image = context
                .image()
                .as_ref()
                .map(|image| {
                    format!(
                        r#"<meta property="og:image" content="{}" />"#,
                        encode_double_quoted_attribute(image)
                    )
                })
                .unwrap_or_default();

            let alternates = options
                .and_then(|options| options.alternates)
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
    <link rel="stylesheet" type="text/css" href="https://fonts.googleapis.com/css2?family=Inter:slnt,wght@-10..0,100..900&family=IBM+Plex+Mono:wght@400&display=swap" />
    <link rel="stylesheet" type="text/css" href="/~static/themes/default.css" />
    <link rel="stylesheet" type="text/css" href="/~static/views/dynamic.css" />
    <script type="module" src="/~static/views/dynamic.js"></script>
    {style}
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
            false => indent(&html)?,
        };

        Ok((html, EncodeInfo::none()))
    }
}

/// Indent HTML
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(html: &str) -> Result<String> {
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
