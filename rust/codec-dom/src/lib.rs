use std::path::Path;

use itertools::Itertools;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use tokio::fs::{create_dir_all, write};

use stencila_codec::{
    Codec, CodecSupport, EncodeInfo, EncodeOptions, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::{Node, NodeType},
};
use stencila_codec_dom_trait::{
    DomCodec as DomCodecTrait, DomEncodeContext,
    html_escape::{encode_double_quoted_attribute, encode_safe},
};
use stencila_codec_text_trait::to_text;
use stencila_node_media::{embed_media, extract_media};
use stencila_version::STENCILA_VERSION;

/// A codec for DOM HTML
pub struct DomCodec;

#[async_trait]
impl Codec for DomCodec {
    fn name(&self) -> &str {
        "dom"
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Dom | Format::Html => CodecSupport::NoLoss,
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
        if let Some(media) = options
            .as_ref()
            .and_then(|opts| opts.extract_media.as_ref())
        {
            let mut copy = node.clone();
            extract_media(
                &mut copy,
                options.as_ref().and_then(|opts| opts.to_path.as_deref()),
                media,
            )?;
            encode(&copy, options)
        } else if options
            .as_ref()
            .and_then(|opts| opts.embed_media)
            .unwrap_or_default()
        {
            let mut copy = node.clone();
            embed_media(
                &mut copy,
                options.as_ref().and_then(|opts| opts.from_path.as_deref()),
            )?;
            encode(&copy, options)
        } else {
            encode(node, options)
        }
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let mut options = options.unwrap_or_default();
        if options.standalone.is_none() {
            options.standalone = Some(true);
        }
        if !options.embed_media.unwrap_or_default() && options.extract_media.is_none() {
            options.extract_media = Some(path.with_extension("media"));
        }
        options.to_path = Some(path.to_path_buf());

        let (html, info) = self.to_string(node, Some(options)).await?;

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }
        write(&path, html).await?;

        Ok(info)
    }
}

/// Encode a node to DOM HTML with options
fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<(String, EncodeInfo)> {
    // Encode to DOM HTML
    let mut context = DomEncodeContext::new();
    node.to_dom(&mut context);

    // Add the root attribute to the root node (the first opening tag)
    let mut node_html = context.content();
    if let Some(pos) = node_html.find('>') {
        node_html.insert_str(pos, " root");
    }

    // Get any CSS defined in the content (e.g. Tailwind usage, or raw CSS blocks)
    // This needs to be inserted at the top of the root node
    // (for diffing and Morphdom to work it can not go before)
    let css = context.css();
    if !css.is_empty() {
        let css = normalize_css(&css);
        if let Some(pos) = node_html.find('>') {
            node_html.insert_str(pos + 1, &["<style>", &css, "</style>"].concat());
        }
    }

    let standalone = options
        .as_ref()
        .and_then(|options| options.standalone)
        .unwrap_or(false);
    let html = if !standalone {
        node_html
    } else {
        let node_type = node.node_type();

        let node_title = match node {
            Node::Article(article) => article.title.as_ref().map(to_text),
            Node::Prompt(prompt) => Some(to_text(&prompt.title)),
            _ => None,
        };

        let node_description = match node {
            Node::Article(article) => article
                .options
                .description
                .as_ref()
                .map(|cord| cord.to_string()),
            Node::Prompt(prompt) => Some(prompt.description.to_string()),
            _ => None,
        };

        let mut extra_head = options
            .as_ref()
            .and_then(|options| options.alternates.clone())
            .iter()
            .flatten()
            .map(|(typ, path)| format!(r#"<link rel="alternate" type="{typ}" href="{path}" />"#))
            .join("\n    ");

        if let Some(image) = context.image().as_ref() {
            let base_url = options
                .as_ref()
                .and_then(|options| options.base_url.as_deref())
                .unwrap_or_default();
            extra_head.push_str(&format!(
                r#"<meta property="og:image" content="{}" />"#,
                encode_double_quoted_attribute(&format!("{base_url}/{image}"))
            ));
        };

        let extra_head = (!extra_head.is_empty()).then_some(extra_head);

        // The URL prefix for Stencila's web distribution
        let web_base = format!("https://stencila.io/web/v{STENCILA_VERSION}");

        // During development (e.g. when generating PDFs via HTML) it can be useful to
        // use a local development version of the web assets. To do so, uncomment the
        // next line and run `cargo run --bin stencila serve --cors permissive`
        //#[cfg(debug_assertions)]
        //let web_base = format!("http://localhost:9000/~static/dev");

        let theme = options
            .as_ref()
            .and_then(|options| options.theme.as_deref())
            .unwrap_or("stencila");

        let view = options
            .as_ref()
            .and_then(|options| options.view.as_deref())
            .unwrap_or("static");

        standalone_html(
            String::new(),
            node_type,
            node_title,
            node_description,
            extra_head,
            node_html,
            web_base,
            theme,
            view,
        )
    };

    let compact = options
        .as_ref()
        .and_then(|options| options.compact)
        .unwrap_or(true);
    let html = match compact {
        true => html,
        false => indent_html(&html)?,
    };

    Ok((html, EncodeInfo::none()))
}

/// Generate standalone DOM HTML for a document with theme and view
///
/// This is exposed as a public function for user by the `stencila-server` crate
/// so that there is a single, optimized implementation.
pub fn standalone_html(
    doc_id: String,
    node_type: NodeType,
    node_title: Option<String>,
    node_description: Option<String>,
    extra_head: Option<String>,
    node_html: String,
    web_base: String,
    theme: &str,
    view: &str,
) -> String {
    let title = node_title.as_ref().map_or_else(
        || "Stencila Document".to_string(),
        |title| encode_safe(&title).to_string(),
    );

    let mut html = format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <title>{title}</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta property="og:type" content="{node_type}" />
    <link rel="icon" type="image/png" href="{web_base}/images/favicon.png">"#
    );

    // OpenGraph Title
    if let Some(title) = &node_title {
        let escaped = encode_double_quoted_attribute(title);
        html.push_str(&format!(
            r#"
    <meta property="og:title" content="{escaped}" />"#,
        ))
    }

    // HTML and OpenGraph description
    if let Some(description) = &node_description {
        let escaped = encode_double_quoted_attribute(description);
        html.push_str(&format!(
            r#"
    <meta property="description" content="{escaped}" />
    <meta property="og:description" content="{escaped}" />"#,
        ))
    }

    // View fonts
    if !matches!(view, "none" | "paged") {
        html.push_str(&format!(
            r#"
    <link rel="preconnect" href="https://fonts.googleapis.com" crossorigin>
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,300..700;1,14..32,300..700&family=IBM+Plex+Mono:ital,wght@0,400;0,500;0,600;1,400&display=swap" rel="stylesheet">"#
        ))
    }

    // Color scheme initialization
    if !matches!(view, "none" | "paged") {
        html.push_str(&format!(
            r#"
    <script type="module" src="{web_base}/themes/init.js"></script>"#
        ))
    }

    // View CSS
    if !matches!(view, "none" | "paged") && theme != "none" {
        html.push_str(&format!(
            r#"
    <link rel="stylesheet" type="text/css" href="{web_base}/views/{view}.css">"#
        ))
    }

    // View Javascript
    if view != "none" {
        html.push_str(&format!(
            r#"
    <script type="module" src="{web_base}/views/{view}.js"></script>"#
        ))
    }

    // Theme CSS (the data-theme-link is needed for theme switching)
    if theme != "none" {
        html.push_str(&format!(
            r#"
    <link data-theme-link rel="stylesheet" type="text/css" href="{web_base}/themes/{theme}.css">"#
        ));
    }

    // Extra <head> tags
    if let Some(extra_head) = extra_head {
        html.push_str(&extra_head);
    }

    html.push_str(
        r#"
  </head>
  <body>
    "#,
    );

    // Body content
    if view != "none" {
        // Wrap the root's HTML in the view element
        html.push_str(&format!(
            r#"<stencila-{view}-view view={view} doc={doc_id} type={node_type}>{node_html}</stencila-{view}-view>"#
        ));
    } else {
        // No wrapping element if view is none
        html.push_str(&node_html);
    };

    html.push_str(
        r#"
  </body>
</html>"#,
    );

    html
}

/// Indent HTML
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent_html(html: &str) -> Result<String> {
    use quick_xml::{Reader, Writer, events::Event};

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
