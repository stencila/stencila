use std::{any::type_name, collections::HashMap};

use html_escape::{encode_double_quoted_attribute, encode_safe};

use codec::{
    common::{
        eyre::Result,
        inflector::cases::{camelcase::to_camel_case, kebabcase::to_kebab_case},
        once_cell::sync::Lazy,
        serde, serde_json,
        strum::AsRefStr,
        tracing,
    },
    EncodeOptions,
};
use server_next::statics::get_static_bytes;
use stencila_schema::*;

/// Encode a `Node` to a HTML document
pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
    let options = options.unwrap_or_default();

    let html = encode_root(node, Some(options.clone()));

    let html = if options.standalone {
        wrap_standalone(&html, options, "", "")
    } else {
        html
    };

    Ok(html)
}

/// Generate the HTML fragment for a root node
///
/// This function is used when translating a `Operation` (where any value of
/// the operation is a `Node` and the operation is applied to a `Node`) to a `DomOperation`
/// (where any value is either a HTML or JSON string and the operation is applied to a browser DOM).
pub fn encode_root(node: &Node, options: Option<EncodeOptions>) -> String {
    let EncodeOptions {
        bundle, compact, ..
    } = options.unwrap_or_default();

    let context = EncodeContext {
        root: node,
        bundle,
        ..Default::default()
    };
    let html = node.to_html(&context);

    // Add the `data-root` attribute.
    // This is currently used in `themes` (for CSS scope) and in `web` (for address resolution).
    // This is a bit hacky and there may be a better approach. Or we may find
    // a way of avoid this entirely.
    let html = html.replacen(' ', " data-root ", 1);

    if compact {
        html
    } else {
        indent(&html).unwrap_or(html)
    }
}

/// Indent generated HTML
///
/// Originally based on https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
fn indent(html: &str) -> Result<String> {
    use quick_xml::events::Event;
    use quick_xml::{Reader, Writer};

    let mut buf = Vec::new();

    let mut reader = Reader::from_str(html);
    reader.trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        match reader.read_event(&mut buf)? {
            Event::Eof => break,
            event => writer.write_event(event)?,
        };
        buf.clear();
    }

    Ok(std::str::from_utf8(&*writer.into_inner())?.to_string())
}

/// Wrap generated HTML so that it is standalone
pub fn wrap_standalone(html: &str, options: EncodeOptions, title: &str, extra_css: &str) -> String {
    let title = if title.is_empty() { "Untitled" } else { title };
    let theme = options.theme.unwrap_or_else(|| "stencila".to_string());

    // Get the theme CSS
    let theme_css =
        get_static_bytes(&format!("themes/themes/{theme}/styles.css")).unwrap_or_default();
    let theme_css = String::from_utf8_lossy(&theme_css);

    let components = match options.components {
        true => {
            r#"
        <script src="https://unpkg.com/@stencila/components/dist/stencila-components/stencila-components.esm.js" type="module"></script>
        <script src="https://unpkg.com/@stencila/components/dist/stencila-components/stencila-components.js" nomodule=""></script>
            "#
        }
        false => "",
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>{title}</title>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <style>
            {theme_css}
        </style>
        <style>
            {extra_css}
        </style>
        {components}
    </head>
    <body>
        {html}
    </body>
</html>"#,
        title = title,
        theme_css = theme_css,
        extra_css = extra_css,
        components = components,
        html = html
    );

    #[cfg(skip)]
    {
        // This can be useful during debugging to preview the HTML
        use std::io::Write;
        std::fs::File::create("temp.html")
            .expect("Unable to create file")
            .write_all(html.as_bytes())
            .expect("Unable to write data");
    }

    html
}

/// The encoding context.
///
/// Used by child nodes to retrieve necessary information about the
/// parent nodes when rendering themselves.
pub struct EncodeContext<'a> {
    /// The root node being encoded
    pub root: &'a Node,

    /// Whether <img>, <audio> and <video> elements should use dataURIs
    pub bundle: bool,

    /// Whether currently within inline content
    pub inline: bool,

    /// The mode for user interaction with node Web Components
    pub mode: EncodeMode,
}

impl<'a> Default for EncodeContext<'a> {
    fn default() -> Self {
        EncodeContext {
            root: &Node::Null(Null {}),
            bundle: false,
            inline: false,
            mode: EncodeMode::Write,
        }
    }
}

#[derive(Clone, Copy, AsRefStr)]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
pub enum EncodeMode {
    Read,
    Run,
    Write,
}

/// Trait for encoding a node as HTML
pub trait ToHtml {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem("span", &[attr("class", "unsupported")], "<not implemented>")
    }

    fn to_attrs(&self, _context: &EncodeContext) -> Vec<String> {
        Vec::new()
    }
}

/// Create an empty string
///
/// Just used to make it clear that not encoding anything to HTML (usually
/// because a property is missing).
fn nothing() -> String {
    String::new()
}

/// Encode a HTML element
///
/// Use this function for creating HTML strings for elements.
/// This, and other functions below, us slice `concat`, rather than `format!`
/// for performance (given that HTML generation may be done on every, or nearly every, keystroke).
fn elem(name: &str, attrs: &[String], content: &str) -> String {
    let attrs = attrs.iter().fold(String::new(), |mut attrs, attr| {
        if !attr.is_empty() {
            if !attrs.is_empty() {
                attrs.push(' ');
            }
            attrs.push_str(attr);
        }
        attrs
    });
    [
        "<",
        name,
        if attrs.is_empty() { "" } else { " " },
        &attrs,
        ">",
        content,
        "</",
        name,
        ">",
    ]
    .concat()
}

/// Encode an "empty" HTML element
///
/// An empty (a.k.a self-closing) element has no closing tag.
/// See https://developer.mozilla.org/en-US/docs/Glossary/Empty_element
fn elem_empty(name: &str, attrs: &[String]) -> String {
    [
        "<",
        name,
        if attrs.is_empty() { "" } else { " " },
        attrs.join(" ").trim(),
        "/>",
    ]
    .concat()
}

/// Encode a `<meta>` attribute
///
/// This is used to encode simple, usually string, properties of nodes as HTML Microdata
/// when the property should not be visible, or is represented some other way.
fn elem_meta(name: &str, content: &str) -> String {
    elem_empty("meta", &[attr_itemprop(name), attr("content", content)])
}

/// Encode an optional property as an element
///
/// If the property is `None` then the element will be empty but will act
/// as a placeholder for if/when the property is set.
/// By having placeholders the order of optional properties in the HTML tree
/// can be consistent when they are added and removed.
fn elem_placeholder<T: ToHtml>(
    name: &str,
    attrs: &[String],
    content: &Option<T>,
    context: &EncodeContext,
) -> String {
    elem(
        name,
        attrs,
        &match content {
            Some(content) => content.to_html(context),
            None => nothing(),
        },
    )
}

/// Encode a HTML element attribute, ensuring that the value is escaped correctly
fn attr(name: &str, value: &str) -> String {
    [
        &to_kebab_case(name),
        "=\"",
        encode_double_quoted_attribute(&value).as_ref(),
        "\"",
    ]
    .concat()
}

/// Encode a boolean attribute (a flag; does not have a value)
///
/// Will ensure that the name is camelCased.
fn attr_bool(name: &str) -> String {
    to_kebab_case(name)
}

/// Encode one of the attributes used to identify a property of a Stencila node
///
/// Will ensure that the name is camelCased.
fn attr_prop(name: &str) -> String {
    if name.is_empty() {
        "".to_string()
    } else {
        attr("data-prop", &to_camel_case(name))
    }
}

/// A mapping of type and property names to their `itemtype` or `itemprop` values
static IDS: Lazy<HashMap<String, String>> = Lazy::new(|| {
    stencila_schema::IDS
        .iter()
        .map(|(name, id)| (name.to_string(), id.to_string()))
        .collect()
});

/// Encode the "itemtype" attribute of an HTML element
///
/// Prefer to use `attr_itemtype::<Type>` but use this when the
/// itemtype should differ from the Rust type name.
/// Note: there should always be a sibling "itemscope" attribute on the
/// element so that is always added.
fn attr_itemtype_str(name: &str) -> String {
    if name.is_empty() {
        return "".to_string();
    }
    let itemtype = match IDS.get(name) {
        Some(url) => url,
        _ => name,
    };
    [&attr("itemtype", itemtype), " itemscope"].concat()
}

/// Encode the "itemtype" attribute of an HTML element using the type of node
fn attr_itemtype<Type>() -> String {
    let name = type_name::<Type>();
    let name = if let Some(name) = name.strip_prefix("stencila_schema::types::") {
        name
    } else {
        tracing::error!("Unexpected type: {}", name);
        name
    };
    attr_itemtype_str(name)
}

/// Encode a "itemprop" attribute of an HTML element
///
/// Will ensure that the itemprop value is (a) translated to a valid itemprop
/// based on the schema (b) is camelCased.
fn attr_itemprop(itemprop: &str) -> String {
    if itemprop.is_empty() {
        return "".to_string();
    }
    let itemprop = IDS
        .get(itemprop)
        .map_or_else(|| itemprop, |itemprop| itemprop);
    let itemprop = to_camel_case(itemprop);
    attr("itemprop", &itemprop)
}

/// Encode a node `id` as the "id" attribute of an HTML element
#[allow(clippy::box_collection)]
fn attr_id(id: &Option<Box<String>>) -> String {
    match id.as_deref() {
        Some(id) => attr("id", id),
        None => "".to_string(),
    }
}

/// Encode the "slot" attribute of an HTML
///
/// Used for nodes that are represented in HTML using a custom Web Component.
/// Not to be confused with the Stencila `Address` slot which will often have the
/// same value but which will be encoded as a "data-itemprop" if the element is not a Web Component.
fn attr_slot(name: &str) -> String {
    attr("slot", name)
}

/// Encode a property as both an attribute and a <meta> element
fn attr_and_meta(name: &str, value: &str) -> (String, String) {
    (attr(name, value), elem_meta(name, value))
}

/// Encode an optional property as both an attribute and a <meta> element
///
/// If the property value is `None` returns a pair of empty strings.
fn attr_and_meta_opt(name: &str, value: Option<String>) -> (String, String) {
    match value {
        Some(value) => attr_and_meta(name, &value),
        None => (nothing(), nothing()),
    }
}

/// Encode a node as JSON
///
/// Several of the below implementations use this, mainly as a placeholder,
/// until a complete implementation is finished. Ensures that the JSON is
/// properly escaped
fn json(node: &impl serde::Serialize) -> String {
    encode_safe(&serde_json::to_string(node).unwrap_or_default()).to_string()
}

/// Encode a node as indented (pretty) JSON
#[allow(dead_code)]
fn json_pretty(node: &impl serde::Serialize) -> String {
    encode_safe(&serde_json::to_string_pretty(node).unwrap_or_default()).to_string()
}

/// Iterate over a slice of nodes, call a string generating function on each item,
/// and concatenate the strings
pub fn concat<T, F>(slice: &[T], func: F) -> String
where
    F: FnMut(&T) -> String,
{
    slice.iter().map(func).collect::<Vec<String>>().concat()
}

/// Iterate over a slice of nodes, calling `to_html` on each item, and concatenate
pub fn concat_html<T: ToHtml>(slice: &[T], context: &EncodeContext) -> String {
    concat(slice, |item| item.to_html(context))
}

/// Iterate over a slice of nodes, call a string generating function on each item,
/// and join using a separator
#[allow(dead_code)]
pub fn join<T, F>(slice: &[T], func: F, sep: &str) -> String
where
    F: FnMut(&T) -> String,
{
    slice.iter().map(func).collect::<Vec<String>>().join(sep)
}

/// Iterate over a slice of nodes, calling `to_html` on each item, and join using a separator
#[allow(dead_code)]
pub fn join_html<T: ToHtml>(slice: &[T], context: &EncodeContext, sep: &str) -> String {
    join(slice, |item| item.to_html(context), sep)
}

mod blocks;
mod data;
mod generics;
mod inlines;
mod math;
mod nodes;
mod others;
mod primitives;

#[allow(clippy::deprecated_cfg_attr)]
mod works;

#[cfg(test)]
mod tests {
    use codec::common::{eyre::bail, tokio};
    use serde_json::json;
    use test_snaps::{
        insta::assert_display_snapshot, snapshot_fixtures_content, snapshot_fixtures_nodes,
    };
    use test_utils::{assert_json_eq, home, skip_ci, skip_ci_os, skip_slow};

    use crate::decode::decode;

    use super::*;

    /// Encode the node fixtures
    #[test]
    fn encode_nodes() {
        snapshot_fixtures_nodes("nodes/*.json", |node| {
            let html = encode(
                &node,
                Some(EncodeOptions {
                    compact: false,
                    ..Default::default()
                }),
            )
            .unwrap();
            assert_display_snapshot!(html);
        });
    }

    /// Encode the HTML fragment fixtures (involves decoding them first)
    #[test]
    fn encode_html_fragments() {
        snapshot_fixtures_content("fragments/html/*.html", |content| {
            let decoded = decode(content, None).expect("Unable to decode");
            let encoded = encode(
                &decoded,
                Some(EncodeOptions {
                    compact: false,
                    ..Default::default()
                }),
            )
            .unwrap();
            assert_display_snapshot!(encoded);
        });
    }

    /// Validate HTML against https://validator.github.io/validator/
    ///
    /// To run locally using the validator's Docker image:
    ///
    ///  docker run -it --rm -p 8888:8888 ghcr.io/validator/validator
    ///  RUN_SLOW_TESTS=1 HTML_VALIDATOR=http://localhost:8888 cargo test
    ///
    /// See https://github.com/validator/validator/wiki/Service-%C2%BB-Input-%C2%BB-POST-body
    /// for more on the API.
    #[tokio::test]
    async fn nu_validate() -> Result<()> {
        if skip_slow() || skip_ci("The https://validator.w3.org/nu/ service can be offline causing CI to fail") || skip_ci_os("windows", "Failed with error: The filename, directory name, or volume label syntax is incorrect. (os error 123)") {
            return Ok(());
        }

        // Read the existing snapshot
        // We only do this for one, kitchen sink like, snapshot.
        let html = std::fs::read_to_string(
            home().join("rust/codec-html/src/snapshots/encode_html_fragments@heading.html.snap"),
        )?;
        let decoded = decode(&html, None)?;
        let html = encode(
            &decoded,
            Some(EncodeOptions {
                standalone: true,
                compact: false,
                ..Default::default()
            }),
        )?;

        // Make the POST request
        let url = if let Ok(url) = std::env::var("HTML_VALIDATOR") {
            url
        } else {
            "https://validator.w3.org/nu/".to_string()
        };
        let client = reqwest::Client::new();
        let response = client
            .post([&url, "?out=json"].concat())
            .header("Content-Type", "text/html; charset=UTF-8")
            .header("Accept", "application/json")
            .header(
                "User-Agent",
                "Stencila tests (https://github.com/stencila/stencila/)",
            )
            .body(html)
            .send()
            .await?;

        // If the response is a server error (e.g. 503 Service Unavailable) then warn but do not fail
        let is_server_error = response.status().is_server_error();
        match response.error_for_status() {
            Ok(response) => {
                let result: serde_json::Value = response.json().await?;
                assert_json_eq!(result, json!({"messages": []}));
            }
            Err(error) => {
                if is_server_error {
                    eprintln!("https://validator.w3.org/nu/ server error: {:}", error)
                } else {
                    bail!(error)
                }
            }
        };

        Ok(())
    }
}
