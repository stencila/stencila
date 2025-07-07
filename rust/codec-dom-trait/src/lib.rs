//! Provides the `DomCodec` trait for generating HTML for the
//! browser DOM for Stencila Schema nodes

use std::path::PathBuf;

use html_escape::{encode_safe, encode_single_quoted_attribute};

use common::{
    inflector::Inflector, itertools::Itertools, serde::Serialize, serde_json,
    smart_default::SmartDefault,
};
use node_id::NodeId;
use node_type::NodeType;

pub use codec_dom_derive::DomCodec;
pub use html_escape;

pub trait DomCodec {
    /// Encode a Stencila Schema node to DOM HTML
    fn to_dom(&self, context: &mut DomEncodeContext);

    /// Encode a Stencila Schema node to a HTML attribute value
    ///
    /// Will generally only be used on simple types (e.g. numbers,
    /// strings, arrays of strings).
    ///
    /// This default implementation serializes the node to JSON.
    /// Should only need to be overridden by atomic types (for perf)
    /// and `String` (to avoid unnecessary double quotes).
    fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext)
    where
        Self: Serialize,
    {
        context.push_attr(name, &serde_json::to_string(self).unwrap_or_default());
    }
}

macro_rules! atom {
    ($type:ty, $name:literal) => {
        impl DomCodec for $type {
            fn to_dom(&self, context: &mut DomEncodeContext) {
                context
                    .enter_elem(concat!("stencila-", $name))
                    .push_text(&self.to_string())
                    .exit_elem();
            }

            fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext) {
                context.push_attr(name, &self.to_string());
            }
        }
    };
}

atom!(bool, "boolean");
atom!(i64, "integer");
atom!(u64, "unsigned-integer");
atom!(f64, "number");

impl DomCodec for String {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_elem("stencila-string")
            .push_text(self)
            .exit_elem();
    }

    fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext) {
        context.push_attr(name, self);
    }
}

impl<T> DomCodec for Box<T>
where
    T: DomCodec,
{
    fn to_dom(&self, context: &mut DomEncodeContext) {
        self.as_ref().to_dom(context)
    }
}

impl<T> DomCodec for Option<T>
where
    T: DomCodec + Serialize,
{
    fn to_dom(&self, context: &mut DomEncodeContext) {
        if let Some(value) = self {
            value.to_dom(context);
        }
    }

    fn to_dom_attr(&self, name: &str, context: &mut DomEncodeContext) {
        if let Some(value) = self {
            value.to_dom_attr(name, context);
        }
    }
}

impl<T> DomCodec for Vec<T>
where
    T: DomCodec,
{
    fn to_dom(&self, context: &mut DomEncodeContext) {
        for item in self.iter() {
            item.to_dom(context);
        }
    }
}

#[derive(SmartDefault)]
pub struct DomEncodeContext {
    /// The DOM HTML content
    content: String,

    /// The node type of ancestors of the current node
    node_types: Vec<NodeType>,

    /// The names of the current stack of HTML elements
    elements: Vec<String>,

    /// The levels and ids of the current stack of `Heading` nodes
    headings: Vec<(i64, NodeId)>,

    /// The CSS content of the document
    css: String,

    /// The URL of the image to use as the Open Graph image (<<meta property="og:image" ...>)
    ///
    /// Currently the first image in the document. In the future, we may allow for another
    /// image to be selected
    image: Option<String>,

    /// Whether encoding to a standalone document
    pub standalone: bool,

    /// The path of the source document
    pub from_path: Option<PathBuf>,

    /// The path of the destination file
    pub to_path: Option<PathBuf>,

    /// The maximum number of rows of a datatable to encode
    #[default = 1000]
    pub max_datatable_rows: usize,
}

impl DomEncodeContext {
    pub fn new(standalone: bool, source_path: Option<PathBuf>, dest_path: Option<PathBuf>) -> Self {
        Self {
            standalone,
            from_path: source_path,
            to_path: dest_path,
            ..Default::default()
        }
    }

    /// Enter an element
    pub fn enter_elem(&mut self, name: &str) -> &mut Self {
        self.content.push('<');
        self.content.push_str(name);
        self.content.push('>');

        self.elements.push(name.to_string());

        self
    }

    /// Enter an element with one or more attributes
    ///
    /// Optimized equivalent of `enter_elem(name).push(attr, value).push(...)`
    pub fn enter_elem_attrs<const N: usize>(
        &mut self,
        name: &str,
        attrs: [(&str, &str); N],
    ) -> &mut Self {
        self.content.push('<');
        self.content.push_str(name);
        for (attr, value) in attrs {
            self.content.push(' ');
            self.content.push_str(attr);
            self.content.push('=');
            self.push_attr_value(value);
        }
        self.content.push('>');

        self.elements.push(name.to_string());

        self
    }

    /// Enter an element for a node
    pub fn enter_node_elem(
        &mut self,
        name: &str,
        node_type: NodeType,
        node_id: NodeId,
    ) -> &mut Self {
        let id = node_id.to_string();
        let depth = self.node_types.len().to_string();
        let ancestors = self
            .node_types
            .iter()
            .map(|node_type| node_type.to_string())
            .join(".");

        self.enter_elem_attrs(
            name,
            [("id", &id), ("depth", &depth), ("ancestors", &ancestors)],
        );
        self.node_types.push(node_type);

        self
    }

    /// Enter a node with the default, custom element for the node type
    pub fn enter_node(&mut self, node_type: NodeType, node_id: NodeId) -> &mut Self {
        let name = ["stencila-", &node_type.to_string().to_kebab_case()].concat();
        self.enter_node_elem(&name, node_type, node_id)
    }

    /// Enter a heading by adding `<stencila-heading-end>` custom elements for
    /// any previous elements that have a level equal to or greater than the
    /// heading being entered into
    pub fn enter_heading(&mut self, level: i64, node_id: NodeId) -> &mut Self {
        while let Some((prev_level, ..)) = self.headings.last() {
            if prev_level < &level {
                break;
            }

            let (.., node_id) = self.headings.pop().expect("checked in parent if");
            self.enter_elem_attrs("stencila-heading-end", [("heading", &node_id.to_string())])
                .exit_elem();
        }

        self.headings.push((level, node_id.clone()));

        self.enter_node(NodeType::Heading, node_id)
    }

    /// Push an attribute onto the current element
    pub fn push_attr(&mut self, name: &str, value: &str) -> &mut Self {
        self.content.pop();

        self.content.push(' ');
        self.content.push_str(name);
        self.content.push('=');
        self.push_attr_value(value);
        self.content.push('>');

        self
    }

    /// Push an attribute value onto the current element
    fn push_attr_value(&mut self, value: &str) {
        if value.is_empty() || value.contains(['"', '\'', ' ', '\t', '\n', '\\', '/', '>', '<']) {
            // Use single quoting (more terse for JSON attributes because inner double
            // quotes do not need escaping)
            let escaped = encode_single_quoted_attribute(value);
            self.content.push('\'');
            self.content.push_str(&escaped);
            self.content.push('\'');
        } else {
            // Value does not contain special chars so does not need quoting
            self.content.push_str(value)
        }
    }

    /// Push a boolean attribute onto the current element
    pub fn push_attr_boolean(&mut self, name: &str) -> &mut Self {
        self.content.pop();
        self.content.push(' ');
        self.content.push_str(name);
        self.content.push('>');

        self
    }

    /// Push the `id`` property of `Entity` nodes
    ///
    /// Uses `_id` as the attribute to avoid conflict with `id` which is
    /// necessary for DOM syncing etc
    pub fn push_id(&mut self, id: &Option<String>) -> &mut Self {
        if let Some(id) = id {
            self.push_attr("_id", id);
        }

        self
    }

    /// Push unescaped HTML onto the content
    pub fn push_html(&mut self, html: &str) -> &mut Self {
        self.content.push_str(html);

        self
    }

    /// Push text onto the content
    pub fn push_text(&mut self, text: &str) -> &mut Self {
        let escaped = encode_safe(text);
        self.content.push_str(&escaped);

        self
    }

    /// Push CSS onto the context
    ///
    ///
    pub fn push_css(&mut self, css: &str) -> &mut Self {
        self.css.push_str(css);

        self
    }

    /// Enter a slot child element
    pub fn enter_slot(&mut self, tag: &str, name: &str) -> &mut Self {
        self.enter_elem_attrs(tag, [("slot", &name.to_kebab_case())])
    }

    /// Exit a slot child element
    pub fn exit_slot(&mut self) -> &mut Self {
        self.exit_elem()
    }

    /// Push a property as a slotted child element
    pub fn push_slot_fn<F>(&mut self, tag: &str, name: &str, func: F) -> &mut Self
    where
        F: Fn(&mut Self),
    {
        self.enter_slot(tag, name);
        func(self);
        self.exit_slot()
    }

    /// Exit a element
    pub fn exit_elem(&mut self) -> &mut Self {
        if let Some(name) = self.elements.pop() {
            self.content.push_str("</");
            self.content.push_str(&name);
            self.content.push('>');
        };

        self
    }

    /// Exit a "void" HTML element e.g <img/>
    pub fn exit_elem_void(&mut self) -> &mut Self {
        if let Some(..) = self.elements.pop() {
            self.content.pop();
            self.content.push_str("/>");
        }

        self
    }

    /// Exit a node
    pub fn exit_node(&mut self) -> &mut Self {
        self.exit_elem();
        self.node_types.pop();

        self
    }

    /// Get the content of the encoding context at completion of encoding
    pub fn content(&mut self) -> String {
        // Use take instead of cloning for performance
        std::mem::take(&mut self.content)
    }

    /// Get the CSS of the encoding context at completion of encoding
    ///
    /// This CSS should be placed in the <head> if standalone (to avoid flash of unstyled content),
    /// or at the top of the root node otherwise.
    pub fn css(&mut self) -> String {
        std::mem::take(&mut self.css)
    }

    /// Set the URL of the image to use in `<meta property="og:image" ...>` tag for the document
    pub fn set_image(&mut self, url: &str) -> &mut Self {
        self.image = Some(url.to_string());

        self
    }

    /// Get the URL of the image to use in `<meta property="og:image" ...>` tag for the document
    pub fn image(&self) -> &Option<String> {
        &self.image
    }

    /// Get the path of the images directory for the context
    pub fn images_dir(&self) -> PathBuf {
        match self.to_path.as_deref() {
            // images directory will be a sibling to the encoded file
            Some(to_path) => PathBuf::from(to_path.to_string_lossy().to_string() + ".images"),
            // images directory will be in the current directory
            None => PathBuf::from("images"),
        }
    }
}
