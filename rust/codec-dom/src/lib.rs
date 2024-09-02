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
use codec_dom_trait::{DomCodec as _, DomEncodeContext};
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

        // Add the root attribute to the root node
        // (the first opening tag)
        let mut dom = context.get_content();
        if let Some(pos) = dom.find('>') {
            dom.insert_str(pos, " root");
        }

        let html = if standalone {
            let title = match node {
                Node::Article(article) => article.title.as_ref().map(to_text),
                _ => None,
            }
            .unwrap_or_else(|| "Unnamed".to_string());

            let alternates = options
                .and_then(|options| options.alternates)
                .iter()
                .flatten()
                .map(|(typ, path)| {
                    format!(r#"<link rel="alternate" type="{typ}" href="{path}" />"#)
                })
                .join("\n    ");

            let style = context.get_style();

            format!(
                r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8"/>
    <title>{title}</title>
    <link rel="icon" type="image/png" href="/~static/images/favicon.png" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="stylesheet" type="text/css" href="https://fonts.googleapis.com/css2?family=Inter:slnt,wght@-10..0,100..900&family=IBM+Plex+Mono:wght@400&display=swap" />
    <link rel="stylesheet" type="text/css" href="/~static/themes/default.css" />
    <link rel="stylesheet" type="text/css" href="/~static/views/dynamic.css" />
    <script type="module" src="/~static/views/dynamic.js"></script>
    {alternates}
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
